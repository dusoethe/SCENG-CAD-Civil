//! First visible workspace for the native SCENG CAD Civil Survey module.

use crate::app::Message;
use iced::widget::{column, container, row, text, Space};
use iced::{Background, Border, Element, Fill, Length, Theme};

fn muted_text(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().background.base.text.scale_alpha(0.66)),
    }
}

fn panel_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.palette().background.weakest.color)),
        border: Border {
            color: theme.palette().background.neutral.color,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn flow_step<'a>(number: &'static str, title: &'static str, body: &'static str) -> Element<'a, Message> {
    row![
        container(text(number).size(13))
            .padding([5, 9])
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(theme.palette().primary.weak.color)),
                border: Border {
                    color: theme.palette().primary.base.color.scale_alpha(0.35),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        column![text(title).size(14), text(body).size(11).style(muted_text)]
            .spacing(3)
            .width(Fill),
    ]
    .spacing(10)
    .align_y(iced::Center)
    .into()
}

pub fn view_window(sizing: crate::ui::modal::ModalSizing) -> Element<'static, Message> {
    let workflow = container(
        column![
            text("Fluxo Survey → Terrain").size(16),
            Space::new().height(5),
            flow_step(
                "01",
                "Dados de campo ou dados abertos",
                "Pontos, códigos, estações, observações e fontes com proveniência.",
            ),
            flow_step(
                "02",
                "Controle topográfico",
                "CRS, datum vertical, poligonais, nivelamento e relatório de qualidade.",
            ),
            flow_step(
                "03",
                "Terreno compartilhado",
                "A SurveyProject entrega fontes controladas ao Terrain, Road Design e Subdivision.",
            ),
        ]
        .spacing(12),
    )
    .padding(16)
    .width(Length::FillPortion(6))
    .style(panel_style);

    let foundation = container(
        column![
            text("Base nativa ativa").size(16),
            Space::new().height(5),
            text("• Projeto e referência geodésica").size(12),
            text("• Campanhas, equipamentos e pontos").size(12),
            text("• Fontes LandXML, SRTM, GeoTIFF e LiDAR").size(12),
            text("• Importação CSV/TXT configurável").size(12),
            text("• Relatórios de qualidade e classificação").size(12),
            Space::new().height(12),
            text("Próxima entrega: persistência no desenho, importadores LandXML/CSV e criação de TIN.")
                .size(11)
                .style(muted_text),
        ]
        .spacing(9),
    )
    .padding(16)
    .width(Length::FillPortion(5))
    .style(panel_style);

    container(
        column![
            text("SCENG CAD Civil Survey").size(22),
            text("Levantamentos, controle geodésico e procedência do terreno.")
                .size(12)
                .style(muted_text),
            Space::new().height(8),
            row![workflow, foundation].spacing(12).width(sizing.width),
        ]
        .spacing(5)
        .padding(18)
        .width(sizing.width)
        .height(sizing.height),
    )
    .width(sizing.width)
    .height(sizing.height)
    .style(|theme: &Theme| container::Style {
        background: Some(Background::Color(theme.palette().background.base.color)),
        ..Default::default()
    })
    .into()
}
