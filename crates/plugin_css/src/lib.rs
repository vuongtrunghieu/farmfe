use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{CssModuleMetaData, Module, ModuleId, ModuleMetaData, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam,
  },
  resource::{
    resource_pot::{CssResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceType,
  },
  swc_common::DUMMY_SP,
  swc_css_ast::Stylesheet,
};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  fs::read_file_utf8,
  script::module_type_from_id,
};

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmPluginCss {}

impl Plugin for FarmPluginCss {
  fn name(&self) -> &str {
    "FarmPluginCss"
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.id);

    if matches!(module_type, ModuleType::Css) {
      let content = read_file_utf8(param.id)?;

      Ok(Some(PluginLoadHookResult {
        content,
        module_type,
      }))
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Module>> {
    if matches!(param.module_type, ModuleType::Css) {
      let module_id = ModuleId::new(&param.id, &context.config.root);
      let css_stylesheet = parse_css_stylesheet(
        &module_id.to_string(),
        &param.content,
        context.meta.css.cm.clone(),
      )?;

      let mut module = Module::new(module_id, param.module_type.clone());
      module.meta = ModuleMetaData::Css(CssModuleMetaData {
        ast: css_stylesheet,
      });

      Ok(Some(module))
    } else {
      Ok(None)
    }
  }

  fn analyze_deps(
    &self,
    _param: &mut PluginAnalyzeDepsHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let module_graph = context.module_graph.read();
      let mut merged_css_ast = Stylesheet {
        span: DUMMY_SP,
        rules: vec![],
      };

      for module_id in resource_pot.modules() {
        let module = module_graph.module(module_id).unwrap();
        let module_css_ast: &Stylesheet = &module.meta.as_css().ast;
        merged_css_ast.rules.extend(module_css_ast.rules.to_vec());
      }

      resource_pot.meta = ResourcePotMetaData::Css(CssResourcePotMetaData {
        ast: merged_css_ast,
      });

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let stylesheet = &resource_pot.meta.as_css().ast;

      let css_code = codegen_css_stylesheet(&stylesheet);

      Ok(Some(vec![Resource {
        name: resource_pot.id.to_string() + ".css",
        bytes: css_code.as_bytes().to_vec(),
        emitted: false,
        resource_type: ResourceType::Css,
        resource_pot: resource_pot.id.clone(),
      }]))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginCss {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}
