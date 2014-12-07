
#![allow(dead_code)]
#![feature(quote)]
#![feature(unboxed_closures)]
#![feature(plugin_registrar)]

extern crate collections;
extern crate syntax;
extern crate rustc;

use syntax::parse::token::{Paren, Colon, CloseDelim, RArrow, OpenDelim, Comma};
use syntax::ast::{TokenTree, Ident, Ty, Block, Item, Arg};
use syntax::parse::common::seq_sep_trailing_disallowed;
use syntax::ext::base::{ExtCtxt, MacResult, MacItems};
use syntax::parse::token::keywords::{Fn};
use syntax::ext::build::AstBuilder;  // trait for expr_uint
use syntax::parse::parser::Parser;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ptr::P;
use syntax::ast;

use std::collections::hash_map::HashMap;

use collections::hash::Hash;

use rustc::plugin::Registry;

pub struct MemFunc<Args, Result> 
    where Args : Clone + Eq + Hash, Result : Clone {
    evals : HashMap<Args, Result>
}

#[plugin_registrar]
pub fn memoize_plugin_registrar(reg: &mut Registry) {
    reg.register_macro("memoize", expand_memoize);
    reg.register_macro("memoize_sync", expand_memoize_sync);
}

//memoize!(fn double(i : int) -> int { 2 * i });
fn expand_memoize(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let (name, params, ret_typ, code) = parse_input(cx, args);
    expand(cx, sp, name, params, ret_typ, code)  
}

//memoize_sync!(fn double(i : int) -> int { 2 * i });
fn expand_memoize_sync(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let (name, params, ret_typ, code) = parse_input(cx, args);
    expand_sync(cx, sp, name, params, ret_typ, code)  
}

fn parse_input(ctxt : &mut ExtCtxt, tt : &[TokenTree]) -> (Ident, Vec<(Ident, P<Ty>)>, P<Ty>, P<Block>) {
    let mut parser = ctxt.new_parser_from_tts(tt);

    parser.expect_keyword(Fn);
    let name = parser.parse_ident();
    parser.expect(&OpenDelim(Paren));

    let params = parser.parse_seq_to_end(
        &CloseDelim(Paren),
        seq_sep_trailing_disallowed(Comma),
        |parser : &mut Parser| -> (Ident, P<Ty>) {
            let arg_name = parser.parse_ident();
            parser.expect(&Colon);
            let arg_typ  = parser.parse_ty(); 
            
            (arg_name, arg_typ)
        }
    );

    parser.expect(&RArrow);

    let ret_typ = parser.parse_ty();    
    let code = parser.parse_block();

    (name, params, ret_typ, code)  
}

fn expand_sync(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>, code : P<Block>) -> Box<MacResult + 'static> {
    let shadow_fn   = expand_shadow_fn(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone(), code.clone());
    let shadow_type = expand_shadow_type(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let shadow_impl = expand_shadow_type_impl(ctxt, name.clone());
    let mem_bk_impl = expand_memoize_backend_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let mem_fn_impl = expand_memoize_fn_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let fn_impl     = expand_fn_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    // let lazy_static_sync = expand_lazy_static_sync(ctxt, name.clone());
    // let static_fn_impl_sync = expand_static_fn_impl_sync(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());

    MacItems::new(vec![
        shadow_fn,
        shadow_type,
        shadow_impl,
        mem_bk_impl,
        mem_fn_impl, 
        fn_impl
        // lazy_static_sync,
        // static_fn_impl_sync
    ].into_iter())
}
fn expand(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>, code : P<Block>) -> Box<MacResult + 'static> {
    let shadow_fn      = expand_shadow_fn(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone(), code.clone());
    let shadow_type    = expand_shadow_type(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let shadow_impl    = expand_shadow_type_impl(ctxt, name.clone());
    let mem_bk_impl    = expand_memoize_backend_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let mem_fn_impl    = expand_memoize_fn_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let fn_impl        = expand_fn_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());
    let lazy_static    = expand_lazy_static(ctxt, name.clone());
    let static_fn_impl = expand_static_fn_impl(ctxt, span.clone(), name.clone(), params.clone(), ret_typ.clone());

    MacItems::new(vec![
        shadow_fn,
        shadow_type,
        shadow_impl, 
        mem_bk_impl,
        mem_fn_impl, 
        fn_impl,
        lazy_static,
        static_fn_impl
    ].into_iter())
}

//fn _double_(num : int) -> int { num * 2 }
fn expand_shadow_fn(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>, code : P<Block>) -> P<Item> {
    let inputs : Vec<Arg> = params.into_iter().map(|(name, typ)| { 
        ctxt.arg(span.clone(), name, typ)
    }).collect();

    let name = shadow_fn_name(name.as_str());

    (quote_item!(ctxt,
        fn $name($inputs) -> $ret_typ $code
    ).unwrap())
}

//struct _double_mem_func {
//  backend : memoirs::HashMapMemoizer<(int,), int>;
//}
fn expand_shadow_type(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>) -> P<Item> {
    let name = shadow_typ_name(name.as_str());
    let typ_tup = args_type(ctxt, span.clone(), params);

    (quote_item!(ctxt,
        struct $name {
            backend : memoirs::HashMapMemoizer<$typ_tup, $ret_typ>
        }
    ).unwrap())   
}

/*
impl _double_mem_func {
    fn new() -> _double_mem_func {
        _double_mem_func {
            backend : memoirs::HashMapMemoizer::new() 
        }
    }
}
*/
fn expand_shadow_type_impl(ctxt : &mut ExtCtxt, name : Ident) -> P<Item> {
    let name = shadow_typ_name(name.as_str());

    (quote_item!(ctxt,
        impl $name {
            fn new() -> $name {
                $name {
                    backend : memoirs::HashMapMemoizer::new() 
                }
            }
        }
    ).unwrap())
}

/*
impl memoirs::MemoizeFnBackend<(int,), int> for _double_mem_func {
    fn get_evaluation(&mut self, args : &(int,)) -> Option<int> {
        self.backend.get_evaluation(args)
    }

    fn store_evaluation(&mut self, args : (int,), result : int) {
        self.backend.store_evaluation(args, result);
    }
}
*/
fn expand_memoize_backend_impl(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>) -> P<Item> {
    let name      = shadow_typ_name(name.as_str());
    let typ_tup   = args_type(ctxt, span.clone(), params);

    (quote_item!(ctxt,
        impl memoirs::MemoizeFnBackend<$typ_tup, $ret_typ> for $name {
            fn get_evaluation(&mut self, args : &$typ_tup) -> Option<$ret_typ> {
                self.backend.get_evaluation(args)
            }

            fn store_evaluation(&mut self, args : $typ_tup, result : $ret_typ) {
                self.backend.store_evaluation(args, result);
            }
        }
    ).unwrap())
}

/*
impl memoirs::MemoizeFn<(int,), int> for _double_mem_func {
    fn call_underlying_fn(&self, args : (int,)) -> int {
        _double_.call(args)
    }
}
*/
fn expand_memoize_fn_impl(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>) -> P<Item> {
    let shadow_fn = shadow_fn_name(name.as_str());
    let name      = shadow_typ_name(name.as_str());
    let typ_tup   = args_type(ctxt, span.clone(), params);

    (quote_item!(ctxt,
        impl memoirs::MemoizeFn<$typ_tup, $ret_typ> for $name {
            fn call_underlying_fn(&self, args : $typ_tup) -> $ret_typ {
                $shadow_fn.call(args)
            }
        }
    ).unwrap())
}

/*
impl Fn<(int,), int> for _double_mem_func {
    extern "rust-call" fn call(&self, args: (int,)) -> int {
        use memoirs::MemoizeFn;

        self.mem_call(args)
    }
}
*/
fn expand_fn_impl(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>) -> P<Item> {
    let name    = shadow_typ_name(name.as_str());
    let typ_tup = args_type(ctxt, span.clone(), params);

    (quote_item!(ctxt,
        impl Fn<$typ_tup, $ret_typ> for $name {
            extern "rust-call" fn call(&self, args: $typ_tup) -> $ret_typ {
                use memoirs::MemoizeFn;
                
                self.mem_call(args)
            }
        }
    ).unwrap())
}

/*
lazy_static! {
  static ref double : _doubleMemFunc = _doubleMemFunc::new();
}
*/

fn expand_lazy_static(ctxt : &mut ExtCtxt, name : Ident) -> P<Item> {
    let shadow_name = shadow_typ_name(name.as_str());

    (quote_item!(ctxt,
        lazy_static! {
            static ref $name : $shadow_name = $shadow_name::new();
        }
    ).unwrap())
}

/*
lazy_static! {
  static ref double : std::sync::Mutex<_doubleMemFunc> = std::sync::Mutex::new(DoubleMemFunc::new());
}
*/
fn expand_lazy_static_sync(ctxt : &mut ExtCtxt, name : Ident) -> P<Item> {
    let shadow_name = shadow_typ_name(name.as_str());

    (quote_item!(ctxt,
        #[phase(plugin)]
        extern crate lazy_static;

        lazy_static! {
            static ref $name : std::sync::Mutex<$shadow_name> = std::sync::Mutex::new($shadow_name::new());
        }
    ).unwrap())
}

/*
impl Fn<(int,), int> for double {
    extern "rust-call" fn call(&self, args: (int,)) -> int {
        self.deref().call(args)
    }
}
*/
fn expand_static_fn_impl(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>) -> P<Item> {
    let typ_tup = args_type(ctxt, span.clone(), params);

    (quote_item!(ctxt,
        impl Fn<$typ_tup, $ret_typ> for $name {
            extern "rust-call" fn call(&self, args: $typ_tup) -> $ret_typ {
                self.deref().call(args)    
            } 
        }
     ).unwrap())
}

/*
impl Fn<(int,), int> for double {
    extern "rust-call" fn call(&self, args: (int,)) -> int {
        let self : &mut double = mem::transmute(self);
        (*self).lock().call_mut(args)
    }
}
*/
fn expand_static_fn_impl_sync(ctxt : &mut ExtCtxt, span : Span, name : Ident, params : Vec<(Ident,P<Ty>)>, ret_typ : P<Ty>) -> P<Item> {
    let typ_tup = args_type(ctxt, span.clone(), params);

    (quote_item!(ctxt,
        impl Fn<$typ_tup, $ret_typ> for $name {
            extern "rust-call" fn call(&self, args: $typ_tup) -> $ret_typ {
                let _self : &mut $name = unsafe { std::mem::transmute(self) };
                (*_self).lock().call_mut(args)
            } 
        }
     ).unwrap())
}

fn shadow_typ_name(func_name : &str) -> Ident {
    token::str_to_ident(format!("_{}MemFunc", func_name).as_slice())
}

fn shadow_fn_name(func_name : &str) -> Ident {
    token::str_to_ident(format!("_{}_", func_name).as_slice())
}

fn args_type(ctxt : &mut ExtCtxt, span : Span, args : Vec<(Ident,P<Ty>)>) -> P<Ty> {
    let typ_tup = ast::TyTup(args.into_iter().map(|(_, typ)| typ).collect());
    ctxt.ty(span, typ_tup)
}   

