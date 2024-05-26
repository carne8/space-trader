use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Symbol {
    sector: Vec<char>,
    system: Option<Vec<char>>,
    waypoint: Option<Vec<char>>,
}

// Serde should serialize "X1-DF55-20250Z" like this:
// Symbol {
//     sector: vec!['X', '1'],
//     system: Some(vec!['D', 'F', '5', '5']),
//     waypoint: Some(vec!['2', '0', '2', '5', '0', 'Z'])
// }
impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SymbolVisitor;

        impl<'de> serde::de::Visitor<'de> for SymbolVisitor {
            type Value = Symbol;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing a sector, system, and waypoint")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut parts = value.split('-');

                let sector = parts.next().unwrap().chars().collect();
                let system = parts.next().map(|s| s.chars().collect());
                let waypoint = parts.next().map(|w| w.chars().collect());

                Ok(Symbol {
                    sector,
                    system,
                    waypoint,
                })
            }
        }

        deserializer.deserialize_str(SymbolVisitor)
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let mut s = String::from_iter(&self.sector);

        if let Some(system) = &self.system {
            let system_string = String::from_iter(system);
            s.push_str("-");
            s.push_str(&system_string);
        }

        if let Some(waypoint) = &self.waypoint {
            let waypoint_string = String::from_iter(waypoint);
            s.push_str("-");
            s.push_str(&waypoint_string);
        }

        serializer.serialize_str(&s)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_iter(&self.sector))?;

        if let Some(system) = &self.system {
            write!(f, "-{}", String::from_iter(system))?;
        }

        if let Some(waypoint) = &self.waypoint {
            write!(f, "-{}", String::from_iter(waypoint))?
        }

        Ok(())
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.sector == other.sector
            && self.system == other.system
            && self.waypoint == other.waypoint
    }
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub account_id: String,
    pub symbol: Symbol,
    pub headquarters: String,
    pub credits: i32,
    pub starting_faction: String,
    pub ship_count: i8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WaypointType {
    Planet,
    GasGiant,
    Moon,
    OrbitalStation,
    JumpGate,
    AsteroidField,
    Asteroid,
    EngineeredAsteroid,
    AsteroidBase,
    Nebula,
    DebrisField,
    GravityWell,
    ArtificialGravityWell,
    FuelStation
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Waypoint {
    pub symbol: Symbol,
    #[serde(rename = "type")]
    pub waypoint_type: WaypointType,
    pub x: i32,
    pub y: i32,
    // pub orbitals: Vec<Orbital>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemType {
    NeutronStar,
    RedStar,
    OrangeStar,
    BlueStar,
    YoungStar,
    WhiteDwarf,
    BlackHole,
    Hypergiant,
    Nebula,
    Unstable
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub symbol: Symbol,
    pub sector_symbol: Symbol,
    #[serde(rename = "type")]
    pub system_type: SystemType,
    pub x: i32,
    pub y: i32,
    pub waypoints: Vec<Waypoint>,
    // pub factions: Vec<Faction>
}
