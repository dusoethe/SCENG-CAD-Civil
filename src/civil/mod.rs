//! Native civil-engineering data owned by SCENG CAD Civil.
//!
//! The contents of this module deliberately do not mirror Civil 3D objects.
//! They are stable application records which may be serialized with a drawing,
//! exchanged through adapters, and consumed by Survey, Terrain, Road Design,
//! Subdivision, Drainage, Utilities, Earthworks, Structures, Quantity Takeoff,
//! and IFC workflows.

pub mod survey;
