// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

#![cfg_attr(docsrs, feature(doc_cfg))]
extern crate proc_macro;

use proc_macro::TokenStream;
use pyo3_macros_backend::{
    build_derive_from_pyobject, build_py_class, build_py_function, build_py_methods,
    build_py_proto, get_doc, process_functions_in_module, py_init, PyClassArgs, PyClassMethodsType,
    PyFunctionAttr,
};
use quote::quote;
use syn::parse_macro_input;

/// A proc macro used to implement Python modules.
///
/// For more on creating Python modules
/// see the [module section of the guide](https://pyo3.rs/main/module.html).
#[proc_macro_attribute]
pub fn pymodule(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);

    let modname = if attr.is_empty() {
        ast.sig.ident.clone()
    } else {
        parse_macro_input!(attr as syn::Ident)
    };

    if let Err(err) = process_functions_in_module(&mut ast) {
        return err.to_compile_error().into();
    }

    let doc = match get_doc(&ast.attrs, None, false) {
        Ok(doc) => doc,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = py_init(&ast.sig.ident, &modname, doc);

    quote!(
        #ast
        #expanded
    )
    .into()
}

/// A proc macro used to implement Python's [dunder methods][1].
///
/// This atribute is required on blocks implementing [`PyObjectProtocol`][2],
/// [`PyNumberProtocol`][3], [`PyGCProtocol`][4] and [`PyIterProtocol`][5].
///
/// [1]: https://docs.python.org/3/reference/datamodel.html#special-method-names
/// [2]: ../class/basic/trait.PyObjectProtocol.html
/// [3]: ../class/number/trait.PyNumberProtocol.html
/// [4]: ../class/gc/trait.PyGCProtocol.html
/// [5]: ../class/iter/trait.PyIterProtocol.html
#[proc_macro_attribute]
pub fn pyproto(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = build_py_proto(&mut ast).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

/// A proc macro used to expose Rust structs as Python objects.
///
/// `#[pyclass]` accepts the following [parameters][2]:
///
/// |  Parameter  |  Description |
/// | :-  | :- |
/// | <span style="white-space: pre">`name = "python_name"`</span> | Sets the name that Python sees this class as. Defaults to the name of the Rust struct. |
/// | <span style="white-space: pre">`freelist = N`</span> |  Implements a [free list][10] of size N. This can improve performance for types that are often created and deleted in quick succession. Profile your code to see whether `freelist` is right for you.  |
/// | `gc`  | Participate in Python's [garbage collection][5]. Required if your type contains references to other Python objects. If you don't (or incorrectly) implement this, contained Python objects may be hidden from Python's garbage collector and you may leak memory. Note that leaking memory, while undesirable, [is safe behavior][7].|
/// | `weakref` | Allows this class to be [weakly referenceable][6]. |
/// | <span style="white-space: pre">`extends = BaseType`</span>  | Use a custom baseclass. Defaults to [`PyAny`][4] |
/// | `subclass` | Allows other Python classes and `#[pyclass]` to inherit from this class.  |
/// | `unsendable` | Required if your struct is not [`Send`][3]. Rather than using `unsendable`, consider implementing your struct in a threadsafe way by e.g. substituting [`Rc`][8] with [`Arc`][9]. By using `unsendable`, your class will panic when accessed by another thread.|
/// | <span style="white-space: pre">`module = "module_name"`</span> |  Python code will see the class as being defined in this module. Defaults to `builtins`. |
///
/// For more on creating Python classes,
/// see the [class section of the guide][1].
///
/// [1]: https://pyo3.rs/main/class.html
/// [2]: https://pyo3.rs/main/class.html#customizing-the-class
/// [3]: std::marker::Send
/// [4]: ../prelude/struct.PyAny.html
/// [5]: https://pyo3.rs/main/class/protocols.html#garbage-collector-integration
/// [6]: https://docs.python.org/3/library/weakref.html
/// [7]: https://doc.rust-lang.org/nomicon/leaking.html
/// [8]: std::rc::Rc
/// [9]: std::sync::Arc
/// [10]: https://en.wikipedia.org/wiki/Free_list
#[proc_macro_attribute]
pub fn pyclass(attr: TokenStream, input: TokenStream) -> TokenStream {
    pyclass_impl(attr, input, PyClassMethodsType::Specialization)
}

/// A proc macro used to expose Rust structs as Python objects.
///
/// `#[pyclass]` accepts the following [parameters][2]:
///
/// |  Parameter  |  Description |
/// | :-  | :- |
/// | <span style="white-space: pre">`name = "python_name"`</span> | Sets the name that Python sees this class as. Defaults to the name of the Rust struct. |
/// | <span style="white-space: pre">`freelist = N`</span> |  Implements a [free list][10] of size N. This can improve performance for types that are often created and deleted in quick succession. Profile your code to see whether `freelist` is right for you.  |
/// | `gc`  | Participate in Python's [garbage collection][5]. Required if your type contains references to other Python objects. If you don't (or incorrectly) implement this, contained Python objects may be hidden from Python's garbage collector and you may leak memory. Note that leaking memory, while undesirable, [is safe behavior][7].|
/// | `weakref` | Allows this class to be [weakly referenceable][6]. |
/// | <span style="white-space: pre">`extends = BaseType`</span> | Use a custom baseclass. Defaults to [`PyAny`][4] |
/// | `subclass` | Allows other Python classes and `#[pyclass]` to inherit from this class.  |
/// | `unsendable` | Required if your struct is not [`Send`][3]. Rather than using `unsendable`, consider implementing your struct in a threadsafe way by e.g. substituting [`Rc`][8] with [`Arc`][9]. By using `unsendable`, your class will panic when accessed by another thread.|
/// | <span style="white-space: pre">`module = "module_name"`</span> |  Python code will see the class as being defined in this module. Defaults to `builtins`. |
///
/// For more on creating Python classes,
/// see the [class section of the guide][1].
///
/// [1]: https://pyo3.rs/main/class.html
/// [2]: https://pyo3.rs/main/class.html#customizing-the-class
/// [3]: std::marker::Send
/// [4]: ../prelude/struct.PyAny.html
/// [5]: https://pyo3.rs/main/class/protocols.html#garbage-collector-integration
/// [6]: https://docs.python.org/3/library/weakref.html
/// [7]: https://doc.rust-lang.org/nomicon/leaking.html
/// [8]: std::rc::Rc
/// [9]: std::sync::Arc
/// [10]: https://en.wikipedia.org/wiki/Free_list
#[proc_macro_attribute]
pub fn pyclass_with_inventory(attr: TokenStream, input: TokenStream) -> TokenStream {
    pyclass_impl(attr, input, PyClassMethodsType::Inventory)
}

/// A proc macro used to expose methods to Python.
///
/// Methods within a `#[pymethods]` block can be annotated with the following:
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | [`#[new]`][4]  | Defines the class constructor, like Python's `__new__` method. |
/// | [`#[getter]`][5] and [`#[setter]`][5] | These define getters and setters, similar to Python's `@property` decorator. This is useful for getters/setters that require computation or side effects; if that is not the case consider using [`#[pyo3(get, set)]`][11] on the struct's field(s).|
/// | [`#[staticmethod]`][6]| Defines the method as a staticmethod, like Python's `@staticmethod` decorator.|
/// | [`#[classmethod]`][7]  | Defines the method as a classmethod, like Python's `@classmethod` decorator.|
/// | [`#[call]`][8]  | Allows Python code to call a class instance as a function, like Python's `__call__` method. |
/// | [`#[classattr]`][9]  | Defines a class variable. |
/// | [`#[args]`][10]  | Define a method's default arguments and allows the function to receive `*args` and `**kwargs`.  |
///
/// For more on creating class methods,
/// see the [class section of the guide][1].
///
/// If the [`multiple-pymethods`][2] feature is enabled, it is possible to implement
/// multiple `#[pymethods]` blocks for a single `#[pyclass]`.
/// This will add a transitive dependency on the [`inventory`][3] crate.
///
/// [1]: https://pyo3.rs/main/class.html#instance-methods
/// [2]: https://pyo3.rs/main/features.html#multiple-pymethods
/// [3]: https://docs.rs/inventory/
/// [4]: https://pyo3.rs/main/class.html#constructor
/// [5]: https://pyo3.rs/main/class.html#object-properties-using-getter-and-setter
/// [6]: https://pyo3.rs/main/class.html#static-methods
/// [7]: https://pyo3.rs/main/class.html#class-methods
/// [8]: https://pyo3.rs/main/class.html#callable-objects
/// [9]: https://pyo3.rs/main/class.html#class-attributes
/// [10]: https://pyo3.rs/main/class.html#method-arguments
/// [11]: https://pyo3.rs/main/class.html#object-properties-using-pyo3get-set
#[proc_macro_attribute]
pub fn pymethods(_: TokenStream, input: TokenStream) -> TokenStream {
    pymethods_impl(input, PyClassMethodsType::Specialization)
}

/// A proc macro used to expose methods to Python.
///
/// Methods within a `#[pymethods]` block can be annotated with the following:
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | [`#[new]`][4]  | Defines the class constructor, like Python's `__new__` method. |
/// | [`#[getter]`][5] and [`#[setter]`][5] | These define getters and setters, similar to Python's `@property` decorator. This is useful for getters/setters that require computation or side effects; if that is not the case consider using [`#[pyo3(get, set)]`][11] on the struct's field(s).|
/// | [`#[staticmethod]`][6]| Defines the method as a staticmethod, like Python's `@staticmethod` decorator.|
/// | [`#[classmethod]`][7]  | Defines the method as a classmethod, like Python's `@classmethod` decorator.|
/// | [`#[call]`][8]  | Allows Python code to call a class instance as a function, like Python's `__call__` method. |
/// | [`#[classattr]`][9]  | Defines a class variable. |
/// | [`#[args]`][10]  | Define a method's default arguments and allows the function to receive `*args` and `**kwargs`.  |
///
/// For more on creating class methods,
/// see the [class section of the guide][1].
///
/// If the [`multiple-pymethods`][2] feature is enabled, it is possible to implement
/// multiple `#[pymethods]` blocks for a single `#[pyclass]`.
/// This will add a transitive dependency on the [`inventory`][3] crate.
///
/// [1]: https://pyo3.rs/main/class.html#instance-methods
/// [2]: https://pyo3.rs/main/features.html#multiple-pymethods
/// [3]: https://docs.rs/inventory/
/// [4]: https://pyo3.rs/main/class.html#constructor
/// [5]: https://pyo3.rs/main/class.html#object-properties-using-getter-and-setter
/// [6]: https://pyo3.rs/main/class.html#static-methods
/// [7]: https://pyo3.rs/main/class.html#class-methods
/// [8]: https://pyo3.rs/main/class.html#callable-objects
/// [9]: https://pyo3.rs/main/class.html#class-attributes
/// [10]: https://pyo3.rs/main/class.html#method-arguments
/// [11]: https://pyo3.rs/main/class.html#object-properties-using-pyo3get-set
#[proc_macro_attribute]
pub fn pymethods_with_inventory(_: TokenStream, input: TokenStream) -> TokenStream {
    pymethods_impl(input, PyClassMethodsType::Inventory)
}

/// A proc macro used to expose Rust functions to Python.
///
/// For more on exposing functions,
/// see the [function section of the guide][1].
///
/// [1]: https://pyo3.rs/main/function.html
#[proc_macro_attribute]
pub fn pyfunction(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(attr as PyFunctionAttr);

    let expanded = build_py_function(&mut ast, args).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_derive(FromPyObject, attributes(pyo3))]
pub fn derive_from_py_object(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as syn::DeriveInput);
    let expanded = build_derive_from_pyobject(&ast).unwrap_or_else(|e| e.to_compile_error());
    quote!(
        #expanded
    )
    .into()
}

fn pyclass_impl(
    attr: TokenStream,
    input: TokenStream,
    methods_type: PyClassMethodsType,
) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attr as PyClassArgs);
    let expanded =
        build_py_class(&mut ast, &args, methods_type).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

fn pymethods_impl(input: TokenStream, methods_type: PyClassMethodsType) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded =
        build_py_methods(&mut ast, methods_type).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}
