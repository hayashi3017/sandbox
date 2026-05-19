//! # 手続きマクロ（Procedural Macros）実装クレート
//!
//! このクレートは学習用の手続きマクロを実装します。
//!
//! ## 手続きマクロの3種類
//!
//! | 種類 | アノテーション | 用途 |
//! |---|---|---|
//! | Deriveマクロ | `#[proc_macro_derive(Name)]` | `#[derive(Name)]`を実装 |
//! | 属性マクロ | `#[proc_macro_attribute]` | `#[attr]`を実装 |
//! | 関数形式マクロ | `#[proc_macro]` | `name!(...)` を実装 |
//!
//! ## TokenStreamの処理
//!
//! 手続きマクロの入出力は`proc_macro::TokenStream`です。
//! 通常は`syn`でパース、`quote`で生成します。

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    Data, DeriveInput, Fields, ItemFn,
};

// ─── #[derive(Describe)] ─────────────────────────────────────────────────────

/// 構造体の各フィールド名と値を文字列にする`describe()`メソッドを生成するderive
///
/// # 使用例
/// ```rust,ignore
/// #[derive(Describe)]
/// struct Person { name: String, age: u32 }
///
/// let p = Person { name: "Alice".into(), age: 30 };
/// assert!(p.describe().contains("name: Alice"));
/// ```
#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => panic!("Describe は structにのみ適用可能"),
    };

    let field_descriptions = match fields {
        Fields::Named(named) => {
            named.named.iter().map(|f| {
                let fname = f.ident.as_ref().unwrap();
                let fname_str = fname.to_string();
                quote! {
                    parts.push(format!("{}: {:?}", #fname_str, self.#fname));
                }
            }).collect::<Vec<_>>()
        }
        Fields::Unnamed(unnamed) => {
            unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                let idx = syn::Index::from(i);
                let idx_str = i.to_string();
                quote! {
                    parts.push(format!("[{}]: {:?}", #idx_str, self.#idx));
                }
            }).collect::<Vec<_>>()
        }
        Fields::Unit => vec![],
    };

    let struct_name = name.to_string();
    let expanded = quote! {
        impl #name {
            pub fn describe(&self) -> String {
                let mut parts: Vec<String> = Vec::new();
                #(#field_descriptions)*
                format!("{}{{ {} }}", #struct_name, parts.join(", "))
            }
        }
    };
    TokenStream::from(expanded)
}

// ─── #[derive(Builder)] ──────────────────────────────────────────────────────

/// ビルダーパターンを自動生成するderive
///
/// 対応する`XxxBuilder`型と`builder()`コンストラクタを生成する。
///
/// # 使用例
/// ```rust,ignore
/// #[derive(Builder)]
/// struct Config { host: String, port: u16 }
///
/// let c = Config::builder().host("localhost".to_string()).port(8080).build().unwrap();
/// ```
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => &n.named,
            _ => panic!("Builder は named struct にのみ適用可能"),
        },
        _ => panic!("Builder は struct にのみ適用可能"),
    };

    let field_names: Vec<_> = fields.iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let field_name_strs: Vec<_> = field_names.iter()
        .map(|n| n.to_string())
        .collect();

    let builder_fields = field_names.iter().zip(field_types.iter()).map(|(n, t)| {
        quote! { #n: Option<#t> }
    });

    let setters = field_names.iter().zip(field_types.iter()).map(|(n, t)| {
        quote! {
            pub fn #n(mut self, val: #t) -> Self {
                self.#n = Some(val);
                self
            }
        }
    });

    let build_fields = field_names.iter().zip(field_name_strs.iter()).map(|(n, s)| {
        quote! {
            #n: self.#n.ok_or_else(|| format!("field '{}' is required", #s))?
        }
    });

    let expanded = quote! {
        #[derive(Default)]
        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name::default()
            }
        }

        impl #builder_name {
            #(#setters)*

            pub fn build(self) -> Result<#name, String> {
                Ok(#name {
                    #(#build_fields),*
                })
            }
        }
    };
    TokenStream::from(expanded)
}

// ─── #[derive(IntoHashMap)] ───────────────────────────────────────────────────

/// 構造体を`HashMap<String, String>`に変換する`into_hashmap()`を生成するderive
#[proc_macro_derive(IntoHashMap)]
pub fn derive_into_hashmap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => &n.named,
            _ => panic!("IntoHashMap は named struct にのみ適用可能"),
        },
        _ => panic!("IntoHashMap は struct にのみ適用可能"),
    };

    let insertions = fields.iter().map(|f| {
        let fname = f.ident.as_ref().unwrap();
        let fname_str = fname.to_string();
        quote! {
            map.insert(#fname_str.to_string(), format!("{:?}", self.#fname));
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn into_hashmap(self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #(#insertions)*
                map
            }
        }
    };
    TokenStream::from(expanded)
}

// ─── #[derive(DefaultNew)] ───────────────────────────────────────────────────

/// `Default`に基づく`new()`を生成するderive
#[proc_macro_derive(DefaultNew)]
pub fn derive_default_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
    TokenStream::from(expanded)
}

// ─── #[logged] 属性マクロ ────────────────────────────────────────────────────

/// 関数の入口と出口をログ出力する属性マクロ
///
/// # 使用例
/// ```rust,ignore
/// #[logged]
/// fn add(a: i32, b: i32) -> i32 { a + b }
/// // add(1, 2)の呼び出し時に "[CALL] add" と "[RETURN] add" を出力
/// ```
#[proc_macro_attribute]
pub fn logged(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let fname = &func.sig.ident;
    let fname_str = fname.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            println!("[CALL] {}", #fname_str);
            let __result = (|| #block)();
            println!("[RETURN] {} -> {:?}", #fname_str, &__result as *const _);
            __result
        }
    };
    TokenStream::from(expanded)
}

// ─── #[retry(times = N)] 属性マクロ ─────────────────────────────────────────

/// 関数が`Err`を返した場合にN回リトライする属性マクロ
///
/// # 使用例
/// ```rust,ignore
/// #[retry(times = 3)]
/// fn might_fail() -> Result<i32, String> { Ok(42) }
/// ```
#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    // リトライ回数をパース（デフォルト3）
    let times: usize = {
        let attr_str = attr.to_string();
        if attr_str.contains("times") {
            attr_str
                .split('=')
                .nth(1)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(3)
        } else {
            3
        }
    };

    let func = parse_macro_input!(item as ItemFn);
    let fname = &func.sig.ident;
    let fname_str = fname.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let mut __attempts = 0;
            loop {
                let __result = (|| #block)();
                __attempts += 1;
                match __result {
                    Ok(v) => return Ok(v),
                    Err(e) if __attempts < #times => {
                        eprintln!("[RETRY] {} failed (attempt {}): {:?}", #fname_str, __attempts, e);
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    };
    TokenStream::from(expanded)
}

// ─── #[measure_time] 属性マクロ ──────────────────────────────────────────────

/// 関数の実行時間を計測する属性マクロ
#[proc_macro_attribute]
pub fn measure_time(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let fname = &func.sig.ident;
    let fname_str = fname.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            let __start = std::time::Instant::now();
            let __result = (|| #block)();
            let __elapsed = __start.elapsed();
            eprintln!("[TIME] {} took {:?}", #fname_str, __elapsed);
            __result
        }
    };
    TokenStream::from(expanded)
}

// ─── count_tokens!(...) 関数形式マクロ ────────────────────────────────────────

/// トークンの個数をカウントする関数形式マクロ
///
/// # 使用例
/// ```rust,ignore
/// assert_eq!(count_tokens!(a b c), 3);
/// assert_eq!(count_tokens!(), 0);
/// ```
#[proc_macro]
pub fn count_tokens(input: TokenStream) -> TokenStream {
    let count = input.into_iter().count();
    let expanded = quote! { #count };
    TokenStream::from(expanded)
}

// ─── sql!(...) 関数形式マクロ ────────────────────────────────────────────────

/// 簡易SQLパーサーマクロ（学習目的）
///
/// `sql!(SELECT field1, field2 FROM table_name WHERE cond)` を構造体に変換。
///
/// # 使用例
/// ```rust,ignore
/// let q = sql!(SELECT name, age FROM users);
/// assert_eq!(q.table, "users");
/// assert_eq!(q.columns, vec!["name", "age"]);
/// ```
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let tokens: Vec<_> = input.into_iter().collect();

    // 簡易パース: SELECT col,... FROM table
    let mut columns = Vec::new();
    let mut table = String::new();
    let mut mode = "start";

    for token in &tokens {
        let s = token.to_string();
        match (mode, s.as_str()) {
            ("start", "SELECT") => mode = "columns",
            ("columns", "FROM") => mode = "table",
            ("columns", ",") => {}
            ("columns", col) => columns.push(col.to_string()),
            ("table", tbl) => {
                table = tbl.to_string();
                mode = "done";
            }
            _ => {}
        }
    }

    let expanded = quote! {
        {
            struct SqlQuery {
                pub table: &'static str,
                pub columns: Vec<&'static str>,
            }
            SqlQuery {
                table: #table,
                columns: vec![#(#columns),*],
            }
        }
    };
    TokenStream::from(expanded)
}

// ─── make_enum!(...) 関数形式マクロ ──────────────────────────────────────────

/// enumと文字列変換を生成する関数形式マクロ
///
/// # 使用例
/// ```rust,ignore
/// make_enum!(Color { Red, Green, Blue });
/// assert_eq!(Color::Red.as_str(), "Red");
/// ```
#[proc_macro]
pub fn make_enum(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    // 簡易パース: Name { V1, V2, ... }
    let tokens: Vec<_> = input2.clone().into_iter().collect();

    let name = tokens.first().map(|t| t.to_string()).unwrap_or_default();
    let name_ident = format_ident!("{}", name);

    // グループ（{...}）の中身を取得
    let variants: Vec<String> = tokens.iter()
        .find_map(|t| {
            if let proc_macro2::TokenTree::Group(g) = t {
                Some(g.stream().into_iter()
                    .filter_map(|t| {
                        if let proc_macro2::TokenTree::Ident(id) = t {
                            Some(id.to_string())
                        } else {
                            None
                        }
                    })
                    .collect())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let variant_idents: Vec<_> = variants.iter()
        .map(|v| format_ident!("{}", v))
        .collect();
    let variant_strs: Vec<&str> = variants.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #name_ident {
            #(#variant_idents),*
        }

        impl #name_ident {
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(Self::#variant_idents => #variant_strs),*
                }
            }
        }

        impl std::fmt::Display for #name_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
    TokenStream::from(expanded)
}
