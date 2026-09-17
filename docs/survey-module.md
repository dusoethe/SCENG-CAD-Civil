# SCENG CAD Civil Survey

`Survey` is a native SCENG CAD Civil module. It owns the provenance,
measurements, coordinate reference system, precision and quality of survey
data. It does not reuse Civil 3D object definitions.

## Domain boundary

`ScengCivilDocument` owns a `SurveyProject` and references generated
`ScengTerrainSurface` objects. Survey produces controlled inputs; Terrain owns
the geometry derived from them (TIN, breaklines, boundaries, contours,
slopes, comparisons and earthworks).

The first implemented domain records are:

- Coordinate reference system, horizontal/vertical datums and epoch;
- campaigns, equipment, stations, observations, points and field codes;
- control networks, traverses, leveling runs, figures and point groups;
- provenance-aware terrain sources and surface classification;
- quality reports and configurable CSV/TXT point import.

## Delivery order

1. Persist `ScengCivilDocument` with SCENG-native drawing metadata and expose
   the Survey workspace;
2. Connect configurable CSV/TXT and LandXML adapters to the Survey project;
3. Build the Terrain engine: TIN, boundaries, breaklines, contours, slope and
   surface comparison;
4. Add geodetic transformations and vertical datum handling;
5. Add an open-data acquisition pipeline: area of interest, provider metadata,
   license, mosaic, reprojection, clipping and source registration;
6. Add GeoTIFF, LAS/LAZ, SHP, GeoJSON, GeoPackage, IFC 4.3, GIS and Civil 3D
   exchange adapters.

Every imported or generated surface retains a classification visible to the
operator: `Preliminar — dados abertos`, `Topográfica — levantamento de campo`,
`Alta resolução — drone/LiDAR`, `Executiva — validada`, or
`As-built — levantamento da obra`.
