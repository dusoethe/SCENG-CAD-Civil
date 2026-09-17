//! Native Survey workspace entry point.
//!
//! This tab intentionally exposes only the working Survey workspace. Import
//! tools will appear here once they write to the persisted SCENG civil model;
//! no ribbon control is shown before its complete workflow exists.

use crate::modules::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};

pub struct SurveyModule;

impl CadModule for SurveyModule {
    fn id(&self) -> &'static str {
        "survey"
    }

    fn title(&self) -> &'static str {
        "Survey"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        GROUPS.get_or_init(|| {
            vec![RibbonGroup {
                title: "Survey",
                tools: vec![RibbonItem::LargeTool(ToolDef {
                    id: "SURVEY",
                    label: "Survey\nWorkspace",
                    icon: IconKind::Svg(include_bytes!("../../../assets/icons/point.svg")),
                    event: ModuleEvent::Command("SURVEY".to_string()),
                })],
            }]
        })
    }
}
