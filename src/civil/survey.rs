//! Survey and terrain provenance model for SCENG CAD Civil.
//!
//! This is the application-domain layer: it retains where data came from and
//! how trustworthy it is. Geometry generation (TIN, contours, volumes and
//! analyses) belongs to the Terrain layer that consumes these records.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CIVIL_DOCUMENT_SCHEMA_VERSION: u32 = 1;

/// Root of the native civil data saved with a SCENG CAD Civil project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScengCivilDocument {
    pub schema_version: u32,
    pub survey: SurveyProject,
    pub terrain_surfaces: Vec<ScengTerrainSurface>,
}

impl Default for ScengCivilDocument {
    fn default() -> Self {
        Self {
            schema_version: CIVIL_DOCUMENT_SCHEMA_VERSION,
            survey: SurveyProject::default(),
            terrain_surfaces: Vec::new(),
        }
    }
}

impl ScengCivilDocument {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            survey: SurveyProject::new(project_name),
            ..Self::default()
        }
    }

    /// Adds a terrain surface that references data owned by the Survey module.
    /// The Terrain module may later populate its TIN, breaklines and analyses.
    pub fn add_terrain_surface(
        &mut self,
        name: impl Into<String>,
        source_ids: Vec<String>,
        classification: TerrainClassification,
    ) -> Result<&mut ScengTerrainSurface, SurveyValidationError> {
        if source_ids.is_empty() {
            return Err(SurveyValidationError::SurfaceNeedsSource);
        }
        if source_ids
            .iter()
            .any(|id| !self.survey.terrain_sources.iter().any(|source| source.id == *id))
        {
            return Err(SurveyValidationError::UnknownTerrainSource);
        }
        let id = next_id("surface", self.terrain_surfaces.iter().map(|surface| &surface.id));
        self.terrain_surfaces.push(ScengTerrainSurface {
            id,
            name: name.into(),
            source_ids,
            classification,
            build_status: TerrainBuildStatus::Draft,
            boundary: None,
            breakline_ids: Vec::new(),
        });
        Ok(self.terrain_surfaces.last_mut().expect("surface was just added"))
    }
}

/// Survey records preserve data lineage and topographic quality, not geometry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SurveyProject {
    pub id: String,
    pub name: String,
    pub coordinate_reference_system: CoordinateReferenceSystem,
    pub campaigns: Vec<SurveyCampaign>,
    pub control_network: ControlNetwork,
    pub traverses: Vec<Traverse>,
    pub leveling_runs: Vec<LevelingRun>,
    pub figures: Vec<SurveyFigure>,
    pub point_groups: Vec<ScengPointGroup>,
    pub terrain_sources: Vec<TerrainSource>,
    pub quality_reports: Vec<QualityReport>,
}

impl Default for SurveyProject {
    fn default() -> Self {
        Self::new("Novo projeto Survey")
    }
}

impl SurveyProject {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: "survey-project-1".to_string(),
            name: name.into(),
            coordinate_reference_system: CoordinateReferenceSystem::default(),
            campaigns: Vec::new(),
            control_network: ControlNetwork::default(),
            traverses: Vec::new(),
            leveling_runs: Vec::new(),
            figures: Vec::new(),
            point_groups: Vec::new(),
            terrain_sources: Vec::new(),
            quality_reports: Vec::new(),
        }
    }

    pub fn add_campaign(&mut self, name: impl Into<String>) -> &mut SurveyCampaign {
        let id = next_id("campaign", self.campaigns.iter().map(|campaign| &campaign.id));
        self.campaigns.push(SurveyCampaign {
            id,
            name: name.into(),
            ..SurveyCampaign::default()
        });
        self.campaigns.last_mut().expect("campaign was just added")
    }

    pub fn campaign_mut(&mut self, campaign_id: &str) -> Option<&mut SurveyCampaign> {
        self.campaigns
            .iter_mut()
            .find(|campaign| campaign.id == campaign_id)
    }

    pub fn add_terrain_source(
        &mut self,
        mut source: TerrainSource,
    ) -> Result<(), SurveyValidationError> {
        if source.id.trim().is_empty() {
            source.id = next_id("source", self.terrain_sources.iter().map(|item| &item.id));
        }
        if self.terrain_sources.iter().any(|item| item.id == source.id) {
            return Err(SurveyValidationError::DuplicateTerrainSourceId(source.id));
        }
        if source.name.trim().is_empty() {
            return Err(SurveyValidationError::TerrainSourceNeedsName);
        }
        self.terrain_sources.push(source);
        Ok(())
    }

    /// Reviews a campaign without silently changing its measurements. Reports
    /// are append-only snapshots, so later workflows can compare revisions.
    pub fn create_quality_report(
        &mut self,
        campaign_id: &str,
    ) -> Result<&QualityReport, SurveyValidationError> {
        let campaign = self
            .campaigns
            .iter()
            .find(|campaign| campaign.id == campaign_id)
            .ok_or_else(|| SurveyValidationError::UnknownCampaign(campaign_id.to_string()))?;
        let report = quality_report_for(campaign);
        self.quality_reports.push(report);
        Ok(self.quality_reports.last().expect("report was just added"))
    }
}

/// Coordinate system, horizontal datum and vertical datum are first-class.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoordinateReferenceSystem {
    pub authority: String,
    pub code: String,
    pub name: String,
    pub horizontal_datum: String,
    pub vertical_datum: String,
    pub linear_unit: String,
    pub coordinate_epoch: Option<String>,
}

impl Default for CoordinateReferenceSystem {
    fn default() -> Self {
        Self {
            authority: "EPSG".to_string(),
            code: String::new(),
            name: "Não definido".to_string(),
            horizontal_datum: String::new(),
            vertical_datum: String::new(),
            linear_unit: "metre".to_string(),
            coordinate_epoch: None,
        }
    }
}

impl CoordinateReferenceSystem {
    pub fn is_defined(&self) -> bool {
        !self.authority.trim().is_empty() && !self.code.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SurveyCampaign {
    pub id: String,
    pub name: String,
    pub started_on: Option<String>,
    pub equipment: Vec<SurveyEquipment>,
    pub stations: Vec<SurveyStation>,
    pub observations: Vec<Observation>,
    pub points: Vec<ScengSurveyPoint>,
    pub revision: u32,
}

impl SurveyCampaign {
    /// Retains only valid finite coordinates. Individual rejected input lines
    /// stay in the import result so operators can correct the original file.
    pub fn append_import(&mut self, imported: &PointImportResult) {
        self.points.extend(imported.points.iter().cloned());
        if !imported.points.is_empty() {
            self.revision = self.revision.saturating_add(1);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SurveyEquipment {
    pub id: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub angular_accuracy_seconds: Option<f64>,
    pub distance_accuracy_mm: Option<f64>,
    pub distance_accuracy_ppm: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SurveyStation {
    pub id: String,
    pub point_id: String,
    pub easting: f64,
    pub northing: f64,
    pub elevation: f64,
    pub instrument_height: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObservationKind {
    Angle,
    Distance,
    Direction,
    GnssVector,
    LevelDifference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Observation {
    pub id: String,
    pub kind: ObservationKind,
    pub from_station_id: String,
    pub to_target_id: String,
    pub measured_value: f64,
    pub standard_deviation: Option<f64>,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: ObservationKind::Distance,
            from_station_id: String::new(),
            to_target_id: String::new(),
            measured_value: 0.0,
            standard_deviation: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PointOrigin {
    FieldSurvey,
    ImportedFile,
    OpenData,
    DroneOrLidar,
    Design,
    AsBuilt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScengSurveyPoint {
    pub id: String,
    pub easting: f64,
    pub northing: f64,
    pub elevation: f64,
    pub code: Option<String>,
    pub description: Option<String>,
    pub origin: PointOrigin,
    pub precision_horizontal_m: Option<f64>,
    pub precision_vertical_m: Option<f64>,
}

impl Default for ScengSurveyPoint {
    fn default() -> Self {
        Self {
            id: String::new(),
            easting: 0.0,
            northing: 0.0,
            elevation: 0.0,
            code: None,
            description: None,
            origin: PointOrigin::ImportedFile,
            precision_horizontal_m: None,
            precision_vertical_m: None,
        }
    }
}

impl ScengSurveyPoint {
    pub fn has_finite_coordinates(&self) -> bool {
        self.easting.is_finite() && self.northing.is_finite() && self.elevation.is_finite()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ControlNetwork {
    pub control_point_ids: Vec<String>,
    pub adjustment_method: Option<String>,
    pub adjusted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Traverse {
    pub id: String,
    pub name: String,
    pub station_ids: Vec<String>,
    pub angular_misclosure_seconds: Option<f64>,
    pub linear_misclosure_m: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LevelingRun {
    pub id: String,
    pub name: String,
    pub benchmark_point_ids: Vec<String>,
    pub closure_error_m: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SurveyFigure {
    pub id: String,
    pub code: String,
    pub point_ids: Vec<String>,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ScengPointGroup {
    pub id: String,
    pub name: String,
    pub point_ids: Vec<String>,
    pub code_filter: Option<String>,
}

/// Provenance and licensing are retained independently from its generated TIN.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainSourceKind {
    DelimitedPoints,
    LandXml,
    Srtm,
    GeoTiff,
    Las,
    Laz,
    Shapefile,
    GeoJson,
    GeoPackage,
    DroneOrLidar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainClassification {
    PreliminaryOpenData,
    TopographicFieldSurvey,
    HighResolution,
    ExecutiveValidated,
    AsBuilt,
}

impl TerrainClassification {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PreliminaryOpenData => "Preliminar — dados abertos",
            Self::TopographicFieldSurvey => "Topográfica — levantamento de campo",
            Self::HighResolution => "Alta resolução — drone/LiDAR",
            Self::ExecutiveValidated => "Executiva — validada",
            Self::AsBuilt => "As-built — levantamento da obra",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSource {
    pub id: String,
    pub name: String,
    pub kind: TerrainSourceKind,
    pub classification: TerrainClassification,
    pub source_uri: Option<String>,
    pub license: Option<String>,
    pub acquired_on: Option<String>,
    pub horizontal_datum: Option<String>,
    pub vertical_datum: Option<String>,
    pub resolution_m: Option<f64>,
    pub vertical_accuracy_m: Option<f64>,
    pub allowed_for_design: bool,
}

impl Default for TerrainSource {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: TerrainSourceKind::DelimitedPoints,
            classification: TerrainClassification::TopographicFieldSurvey,
            source_uri: None,
            license: None,
            acquired_on: None,
            horizontal_datum: None,
            vertical_datum: None,
            resolution_m: None,
            vertical_accuracy_m: None,
            allowed_for_design: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainBuildStatus {
    Draft,
    Current,
    Stale,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScengTerrainSurface {
    pub id: String,
    pub name: String,
    pub source_ids: Vec<String>,
    pub classification: TerrainClassification,
    pub build_status: TerrainBuildStatus,
    pub boundary: Option<SurveyBoundary>,
    pub breakline_ids: Vec<String>,
}

impl Default for ScengTerrainSurface {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            source_ids: Vec::new(),
            classification: TerrainClassification::PreliminaryOpenData,
            build_status: TerrainBuildStatus::Draft,
            boundary: None,
            breakline_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SurveyBoundary {
    pub id: String,
    pub point_ids: Vec<String>,
    pub boundary_type: BoundaryType,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryType {
    #[default]
    Outer,
    Hide,
    DataClip,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualitySeverity {
    Error,
    Warning,
    Information,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct QualityFinding {
    pub severity: QualitySeverity,
    pub code: String,
    pub message: String,
    pub point_id: Option<String>,
}

impl Default for QualityFinding {
    fn default() -> Self {
        Self {
            severity: QualitySeverity::Information,
            code: String::new(),
            message: String::new(),
            point_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct QualityReport {
    pub campaign_id: String,
    pub campaign_revision: u32,
    pub findings: Vec<QualityFinding>,
}

pub fn quality_report_for(campaign: &SurveyCampaign) -> QualityReport {
    let mut ids = BTreeSet::new();
    let mut findings = Vec::new();
    for point in &campaign.points {
        if point.id.trim().is_empty() {
            findings.push(QualityFinding {
                severity: QualitySeverity::Error,
                code: "POINT_ID_MISSING".to_string(),
                message: "Ponto sem identificador.".to_string(),
                point_id: None,
            });
        } else if !ids.insert(point.id.clone()) {
            findings.push(QualityFinding {
                severity: QualitySeverity::Error,
                code: "POINT_ID_DUPLICATE".to_string(),
                message: "Identificador de ponto duplicado.".to_string(),
                point_id: Some(point.id.clone()),
            });
        }
        if !point.has_finite_coordinates() {
            findings.push(QualityFinding {
                severity: QualitySeverity::Error,
                code: "POINT_COORDINATE_INVALID".to_string(),
                message: "Coordenada do ponto não é finita.".to_string(),
                point_id: Some(point.id.clone()),
            });
        }
        if point
            .code
            .as_deref()
            .is_none_or(|code| code.trim().is_empty())
        {
            findings.push(QualityFinding {
                severity: QualitySeverity::Warning,
                code: "POINT_CODE_MISSING".to_string(),
                message: "Ponto sem código de campo.".to_string(),
                point_id: Some(point.id.clone()),
            });
        }
    }
    QualityReport {
        campaign_id: campaign.id.clone(),
        campaign_revision: campaign.revision,
        findings,
    }
}

/// Flexible CSV/TXT import definition. The UI can expose this as a reusable
/// format instead of assuming a vendor-specific column order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DelimitedPointFormat {
    pub delimiter: char,
    pub decimal_separator: char,
    pub has_header: bool,
    pub columns: PointColumnMapping,
}

impl Default for DelimitedPointFormat {
    fn default() -> Self {
        Self {
            delimiter: ',',
            decimal_separator: '.',
            has_header: true,
            columns: PointColumnMapping::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PointColumnMapping {
    pub id: usize,
    pub easting: usize,
    pub northing: usize,
    pub elevation: usize,
    pub code: Option<usize>,
    pub description: Option<usize>,
}

impl Default for PointColumnMapping {
    fn default() -> Self {
        Self {
            id: 0,
            easting: 1,
            northing: 2,
            elevation: 3,
            code: Some(4),
            description: Some(5),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PointImportIssue {
    pub line: usize,
    pub message: String,
}

impl Default for PointImportIssue {
    fn default() -> Self {
        Self {
            line: 0,
            message: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PointImportResult {
    pub points: Vec<ScengSurveyPoint>,
    pub issues: Vec<PointImportIssue>,
}

/// Imports a configurable delimited file without accepting corrupt rows.
/// Quoted CSV fields are intentionally out of scope for this first adapter;
/// the importer reports them for a dedicated robust CSV adapter rather than
/// guessing at a field boundary.
pub fn import_delimited_points(input: &str, format: &DelimitedPointFormat) -> PointImportResult {
    let mut result = PointImportResult::default();
    for (zero_based_line, raw_line) in input.lines().enumerate() {
        let line = zero_based_line + 1;
        if format.has_header && line == 1 {
            continue;
        }
        let raw_line = raw_line.trim();
        if raw_line.is_empty() {
            continue;
        }
        if raw_line.contains('"') {
            result.issues.push(PointImportIssue {
                line,
                message: "Campos entre aspas exigem o importador CSV avançado.".to_string(),
            });
            continue;
        }
        let fields = raw_line.split(format.delimiter).map(str::trim).collect::<Vec<_>>();
        let Some(id) = field(&fields, format.columns.id).filter(|value| !value.is_empty()) else {
            result.issues.push(PointImportIssue {
                line,
                message: "Identificador do ponto ausente.".to_string(),
            });
            continue;
        };
        let parsed = [
            parse_coordinate(field(&fields, format.columns.easting), format.decimal_separator),
            parse_coordinate(field(&fields, format.columns.northing), format.decimal_separator),
            parse_coordinate(field(&fields, format.columns.elevation), format.decimal_separator),
        ];
        let [Some(easting), Some(northing), Some(elevation)] = parsed else {
            result.issues.push(PointImportIssue {
                line,
                message: "Easting, Northing ou elevação inválidos.".to_string(),
            });
            continue;
        };
        result.points.push(ScengSurveyPoint {
            id: id.to_string(),
            easting,
            northing,
            elevation,
            code: format
                .columns
                .code
                .and_then(|index| field(&fields, index))
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            description: format
                .columns
                .description
                .and_then(|index| field(&fields, index))
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            origin: PointOrigin::ImportedFile,
            precision_horizontal_m: None,
            precision_vertical_m: None,
        });
    }
    result
}

fn field<'a>(fields: &'a [&str], index: usize) -> Option<&'a str> {
    fields.get(index).copied()
}

fn parse_coordinate(value: Option<&str>, decimal_separator: char) -> Option<f64> {
    let value = value?.trim();
    let normalized = if decimal_separator == ',' {
        value.replace('.', "").replace(',', ".")
    } else {
        value.replace(',', "")
    };
    let coordinate = normalized.parse::<f64>().ok()?;
    coordinate.is_finite().then_some(coordinate)
}

fn next_id<'a>(prefix: &str, ids: impl Iterator<Item = &'a String>) -> String {
    let used = ids.cloned().collect::<BTreeSet<_>>();
    for suffix in 1.. {
        let candidate = format!("{prefix}-{suffix}");
        if !used.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!("unbounded integer range contains an unused id")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurveyValidationError {
    UnknownCampaign(String),
    DuplicateTerrainSourceId(String),
    TerrainSourceNeedsName,
    SurfaceNeedsSource,
    UnknownTerrainSource,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_semicolon_points_with_brazilian_decimal_separator() {
        let format = DelimitedPointFormat {
            delimiter: ';',
            decimal_separator: ',',
            columns: PointColumnMapping {
                id: 0,
                easting: 1,
                northing: 2,
                elevation: 3,
                code: Some(4),
                description: None,
            },
            ..Default::default()
        };
        let result = import_delimited_points(
            "PONTO;E;N;Z;CODIGO\n101;500.123,50;7.200.456,25;912,75;TN\n102;invalido;7.200.457,00;913,00;TN",
            &format,
        );
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.points[0].id, "101");
        assert!((result.points[0].easting - 500_123.50).abs() < 1.0e-9);
        assert!((result.points[0].northing - 7_200_456.25).abs() < 1.0e-9);
        assert_eq!(result.points[0].code.as_deref(), Some("TN"));
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].line, 3);
    }

    #[test]
    fn quality_report_keeps_duplicate_and_missing_code_findings() {
        let campaign = SurveyCampaign {
            id: "campaign-1".to_string(),
            revision: 4,
            points: vec![
                ScengSurveyPoint {
                    id: "100".to_string(),
                    code: None,
                    ..Default::default()
                },
                ScengSurveyPoint {
                    id: "100".to_string(),
                    code: Some("TN".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let report = quality_report_for(&campaign);
        assert_eq!(report.campaign_id, "campaign-1");
        assert_eq!(report.campaign_revision, 4);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "POINT_ID_DUPLICATE"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "POINT_CODE_MISSING"));
    }

    #[test]
    fn terrain_surface_requires_registered_provenance() {
        let mut civil = ScengCivilDocument::new("Rodovia Norte");
        assert_eq!(
            civil.add_terrain_surface(
                "Existente",
                vec!["source-1".to_string()],
                TerrainClassification::PreliminaryOpenData,
            ),
            Err(SurveyValidationError::UnknownTerrainSource)
        );
        civil
            .survey
            .add_terrain_source(TerrainSource {
                id: "source-1".to_string(),
                name: "SRTM 30 m".to_string(),
                kind: TerrainSourceKind::Srtm,
                classification: TerrainClassification::PreliminaryOpenData,
                allowed_for_design: false,
                ..Default::default()
            })
            .expect("valid terrain source");
        let surface = civil
            .add_terrain_surface(
                "Existente",
                vec!["source-1".to_string()],
                TerrainClassification::PreliminaryOpenData,
            )
            .expect("registered source is accepted");
        assert_eq!(surface.build_status, TerrainBuildStatus::Draft);
        assert_eq!(
            surface.classification.label(),
            "Preliminar — dados abertos"
        );
    }
}
