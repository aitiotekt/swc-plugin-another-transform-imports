use std::collections::HashMap;

use swc_core::{
    ecma::ast::*,
    ecma::visit::{VisitMut, FoldWith, as_folder},
    plugin::{plugin_transform,proxies::TransformPluginProgramMetadata},
};

use serde::{Serialize, Deserialize};
use voca_rs::case::{
    camel_case, 
    kebab_case, 
    pascal_case,
    snake_case, 
    upper_case, 
    upper_first,
    lower_case,
    lower_first,
};
use tracing::debug;
#[macro_use]
extern crate lazy_static;


#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransformMember {
    CamelCase,
    KebabCase,
    DashedCase,
    PascalCase,
    SnakeCase,
    UpperCase,
    UpperFirst,
    LowerCase,
    LowerFirst
}

lazy_static! {
    static ref TRANSFORM_MEMBER_MAPPING: HashMap<TransformMember, fn(&str) -> String> = {
        use TransformMember::*;
        let mut m = HashMap::<TransformMember, fn(&str) -> String>::new();
        m.insert(CamelCase, camel_case);
        m.insert(KebabCase, kebab_case);
        m.insert(PascalCase, pascal_case);
        m.insert(DashedCase, kebab_case);
        m.insert(SnakeCase, snake_case);
        m.insert(UpperCase, upper_case);
        m.insert(UpperFirst, upper_first);
        m.insert(LowerCase, lower_case);
        m.insert(LowerFirst, lower_first);
        m
    }; 
}

fn transform_import_path(transform: &str, member: &Ident, raw: &Str, member_transformers: &[TransformMember]) -> Str {
    let transformed_member = member_transformers.iter().fold(
        member.sym.to_string(),
    |acc, curr| {
        if let Some(f) = TRANSFORM_MEMBER_MAPPING.get(curr) {
            f(&acc)
        } else {
            acc
        }
    });
    debug!("transformed member is {}", transformed_member);
    let replaced = transform
        .replace("${member}", &transformed_member);
    Str {
        span: raw.span.clone(),
        value: replaced.into(),
        raw: None,
    }
}

fn default_true () -> bool {
    true
}

fn default_false () -> bool {
    false
}

fn default_member_transformers () -> Vec<TransformMember> {
    vec![]
}

fn default_style () -> Option<String> {
    None
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransformVisitorSubConfig {
    pub transform: String,
    #[serde(default = "default_false")]
    pub skip_default_conversion: bool,
    #[serde(default = "default_true")]
    pub prevent_full_import: bool,
    #[serde(default = "default_style")]
    pub style: Option<String>,
    #[serde(default = "default_member_transformers")]
    pub member_transformers: Vec<TransformMember>
}

pub type TransformVisitorConfigs = HashMap<String, TransformVisitorSubConfig>;

pub struct TransformVisitor {
    pub configs: TransformVisitorConfigs,
}

impl VisitMut for TransformVisitor {
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html

    fn visit_mut_module_items(&mut self, nodes: &mut Vec<ModuleItem>) {
        let mut transformed_nodes: Vec<ModuleItem> = vec![];

        if self.configs.is_empty() {
            return;
        }

        for node in nodes.iter() {
            match node {
                ModuleItem::ModuleDecl(ref module_decl) => match module_decl {
                    ModuleDecl::Import(ref import_decl) => {

                        let import_decl_value: &str = &import_decl.src.value.to_string();

                        if let Some(config) = self.configs.get(import_decl_value) {
                            let is_default_import_exist = import_decl.specifiers.iter().any(|s| {
                                if let ImportSpecifier::Default(_) | ImportSpecifier::Namespace(_) = s {
                                    true
                                } else {
                                    false
                                }
                            });

                            if is_default_import_exist && config.prevent_full_import {
                                panic!("Import of entire module '{}' not allowed due to preventFullImport setting", import_decl_value);
                            }

                            for spec in &import_decl.specifiers {
                                match spec {
                                    ImportSpecifier::Named(ref import_named_spec) => {
                                        let actural_import_var =
                                            if let Some(ref import_named_spec_name) =
                                                import_named_spec.imported
                                            {
                                                match import_named_spec_name {
                                                    ModuleExportName::Str(s) => {
                                                        Ident::new(s.value.clone(), s.span.clone())
                                                    }
                                                    ModuleExportName::Ident(ident) => ident.clone(),
                                                }
                                            } else {
                                                import_named_spec.local.clone()
                                            };

                                        let transformed_path = transform_import_path(
                                            &config.transform,
                                            &actural_import_var,
                                            &import_decl.src,
                                            &config.member_transformers
                                        );

                                        let new_spec = if config.skip_default_conversion {
                                            spec.clone()
                                        } else {
                                            ImportSpecifier::Default(ImportDefaultSpecifier {
                                                span: import_named_spec.span.clone(),
                                                local: import_named_spec.local.clone(),
                                            })
                                        };

                                        let new_node = ModuleItem::ModuleDecl(ModuleDecl::Import(
                                            ImportDecl {
                                                span: import_decl.span.clone(),
                                                specifiers: vec![new_spec],
                                                src: Box::new(transformed_path),
                                                type_only: import_named_spec.is_type_only,
                                                with: import_decl.with.clone(),
                                                phase: import_decl.phase.clone(),
                                            },
                                        ));

                                        transformed_nodes.push(new_node);

                                        if let Some(ref style_path) = config.style {
                                            let transformed_path = transform_import_path(
                                                &style_path,
                                                &actural_import_var,
                                                &import_decl.src,
                                                &config.member_transformers
                                            );

                                            let style_node = ModuleItem::ModuleDecl(
                                                ModuleDecl::Import(ImportDecl {
                                                    span: import_decl.span.clone(),
                                                    specifiers: vec![],
                                                    src: Box::new(transformed_path),
                                                    type_only: false,
                                                    with: import_decl.with.clone(),
                                                    phase: import_decl.phase.clone(),
                                                }),
                                            );

                                            transformed_nodes.push(style_node);
                                        };
                                    }
                                    _ => {
                                        let new_node = ModuleItem::ModuleDecl(ModuleDecl::Import(
                                            ImportDecl {
                                                span: import_decl.span.clone(),
                                                specifiers: vec![spec.clone()],
                                                src: import_decl.src.clone(),
                                                type_only: import_decl.type_only,
                                                with: import_decl.with.clone(),
                                                phase: import_decl.phase.clone(),
                                            },
                                        ));

                                        transformed_nodes.push(new_node);
                                    }
                                }
                            }
                        } else {
                            transformed_nodes.push(node.clone());
                        }
                    }
                    _ => {
                        transformed_nodes.push(node.clone());
                    }
                },
                n => {
                    transformed_nodes.push(n.clone());
                }
            }
        }

        nodes.clear();
        nodes.extend(transformed_nodes);
    }
}

/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8,
///     ast_ptr_len: i32,
///     config_str_ptr: *const u8,
///     config_str_ptr_len: i32,
///     context_str_ptr: *const u8,
///     context_str_ptr_len: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */
///
/// if plugin need to handle low-level ptr directly. However, there are
/// important steps manually need to be performed like sending transformed
/// results back to host. Refer swc_plugin_macro how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {

    let configs_string_opt = metadata.get_transform_plugin_config();

    let configs: HashMap<String, TransformVisitorSubConfig> = if let Some(ref config_str) = configs_string_opt {
        serde_json::from_str::<HashMap<String, TransformVisitorSubConfig>>(config_str).expect("parse swc-plugin-custom-transform-imports plugin config failed")
    } else {
        HashMap::new()
    };

    program.fold_with(&mut as_folder(TransformVisitor {
        configs,
    }))
}


#[cfg(test)]
mod tests {
    use swc_core::ecma::transforms::testing::test;
    use swc_core::ecma::parser::{Syntax,EsConfig};
    use maplit::hashmap;
    use swc_core::ecma::visit::Fold;
    use super::*;

    fn transform_visitor(configs: TransformVisitorConfigs) -> impl 'static + Fold + VisitMut {
        as_folder(TransformVisitor {
            configs
        })
    }

    fn syntax() -> Syntax {
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        })
    }

    test!(
        syntax(),
        |_| transform_visitor(hashmap!{
            "antd".to_string() => TransformVisitorSubConfig {
                transform: "antd/es/${member}".to_string(),
                skip_default_conversion: false,
                prevent_full_import: true,
                style: Some("antd/es/${member}/style".to_string()),
                member_transformers: vec![TransformMember::DashedCase]
            }
        }),
        base_transform,
        r#"import {MyButton} from "antd";"#,
        r#"import MyButton from "antd/es/my-button";import "antd/es/my-button/style";"#
    );


    test!(
        syntax(),
        |_| transform_visitor(hashmap!{
            "antd".to_string() => TransformVisitorSubConfig {
                transform: "antd/es/${member}".to_string(),
                skip_default_conversion: false,
                prevent_full_import: true,
                style: Some("antd/es/${member}/style".to_string()),
                member_transformers: vec![TransformMember::DashedCase]
            }
        }),
        base_transform_with_alias,
        r#"import {MyButton as NewButton} from "antd";"#,
        r#"import NewButton from "antd/es/my-button";import "antd/es/my-button/style";"#
    );

    test!(
        syntax(),
        |_| transform_visitor(hashmap!{
            "antd".to_string() => TransformVisitorSubConfig {
                transform: "antd/es/${member}".to_string(),
                skip_default_conversion: false,
                prevent_full_import: true,
                style: Some("antd/es/${member}/style".to_string()),
                member_transformers: vec![TransformMember::DashedCase]
            }
        }),
        base_transform_with_others,
        r#"import {MyButton as NewButton} from "abc";"#,
        r#"import {MyButton as NewButton} from "abc";"#
    );

    test!(
        syntax(),
        |_| {
            let configs_str = r#"
            {
                "antd": {
                  "transform": "antd/es/${member}",
                  "skipDefaultConversion": false,
                  "preventFullImport": true,
                  "style": "antd/es/${member}/style",
                  "memberTransformers": ["dashed_case"]
                }
            }
            "#;
            let configs: HashMap<String, TransformVisitorSubConfig> = serde_json::from_str(configs_str)
                .expect("parse swc-plugin-custom-transform-imports plugin config failed");
            transform_visitor(configs)
        },
        base_transform_with_json_config,
        r#"import {MyButton as NewButton} from "antd";"#,
        r#"import NewButton from "antd/es/my-button";import "antd/es/my-button/style";"#
    );
}
