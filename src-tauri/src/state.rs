// state.rs — AppState singleton + Role enum + AuthenticatedUser
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

/// The role enum mirrors the `roles.name` values exactly.
/// Every variant here has a corresponding row in the `roles` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    AgriculturalEngineer,
    BiologicalEngineer,
    DataAnalyst,
    GalacticSecurityHead,
    GalacticSecurityStaff,
    Mathematician,
    Physicist,
    Chemist,
    Biologist,
    Astronaut,
    SettlerCommander,
    CivilEngineer,
    Farmer,
    TemporarySetter,
    SpaceStationSettler,
    Psychiatrist,
    PsychiatristAssistant,
    MedicalStaff,
    HeadOfMedicine,
    HeadOfSanitary,
    InspectorCrew,
    DisposalCrew,
    WastewaterCrew,
    CleanupCrew,
    TransportCrew,
    GeneralDirector,
    TheDirector,
    TheAccountant,
    TheLibrarian,
    TheNomad,
    TheArtificer,
    TheObserver,
    TheWanderer,
    TheTaskmaster,
    TheGuardian,
    TheStatistician,
    TheCoordinator,
    TheOverseer,
    TheAnchorman,
    Administrator,
}

impl Role {
    /// Returns true if this role is any Director-level role
    /// (inherits GeneralDirector voting/meeting capabilities).
    pub fn is_director(&self) -> bool {
        matches!(
            self,
            Role::GeneralDirector
                | Role::TheDirector
                | Role::TheAccountant
                | Role::TheLibrarian
                | Role::TheNomad
                | Role::TheArtificer
                | Role::TheObserver
                | Role::TheWanderer
                | Role::TheTaskmaster
                | Role::TheGuardian
                | Role::TheStatistician
                | Role::TheCoordinator
                | Role::TheOverseer
                | Role::TheAnchorman
                | Role::Administrator
        )
    }

    /// Parses the string value stored in the DB / Redis into a Role.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "AgriculturalEngineer" => Ok(Role::AgriculturalEngineer),
            "BiologicalEngineer" => Ok(Role::BiologicalEngineer),
            "DataAnalyst" => Ok(Role::DataAnalyst),
            "GalacticSecurityHead" => Ok(Role::GalacticSecurityHead),
            "GalacticSecurityStaff" => Ok(Role::GalacticSecurityStaff),
            "Mathematician" => Ok(Role::Mathematician),
            "Physicist" => Ok(Role::Physicist),
            "Chemist" => Ok(Role::Chemist),
            "Biologist" => Ok(Role::Biologist),
            "Astronaut" => Ok(Role::Astronaut),
            "SettlerCommander" => Ok(Role::SettlerCommander),
            "CivilEngineer" => Ok(Role::CivilEngineer),
            "Farmer" => Ok(Role::Farmer),
            "TemporarySetter" => Ok(Role::TemporarySetter),
            "SpaceStationSettler" => Ok(Role::SpaceStationSettler),
            "Psychiatrist" => Ok(Role::Psychiatrist),
            "PsychiatristAssistant" => Ok(Role::PsychiatristAssistant),
            "MedicalStaff" => Ok(Role::MedicalStaff),
            "HeadOfMedicine" => Ok(Role::HeadOfMedicine),
            "HeadOfSanitary" => Ok(Role::HeadOfSanitary),
            "InspectorCrew" => Ok(Role::InspectorCrew),
            "DisposalCrew" => Ok(Role::DisposalCrew),
            "WastewaterCrew" => Ok(Role::WastewaterCrew),
            "CleanupCrew" => Ok(Role::CleanupCrew),
            "TransportCrew" => Ok(Role::TransportCrew),
            "GeneralDirector" => Ok(Role::GeneralDirector),
            "TheDirector" => Ok(Role::TheDirector),
            "TheAccountant" => Ok(Role::TheAccountant),
            "TheLibrarian" => Ok(Role::TheLibrarian),
            "TheNomad" => Ok(Role::TheNomad),
            "TheArtificer" => Ok(Role::TheArtificer),
            "TheObserver" => Ok(Role::TheObserver),
            "TheWanderer" => Ok(Role::TheWanderer),
            "TheTaskmaster" => Ok(Role::TheTaskmaster),
            "TheGuardian" => Ok(Role::TheGuardian),
            "TheStatistician" => Ok(Role::TheStatistician),
            "TheCoordinator" => Ok(Role::TheCoordinator),
            "TheOverseer" => Ok(Role::TheOverseer),
            "TheAnchorman" => Ok(Role::TheAnchorman),
            "Administrator" => Ok(Role::Administrator),
            other => Err(format!("Unknown role: {}", other)),
        }
    }

    /// Returns the database string representation (matches `roles.name`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::AgriculturalEngineer => "AgriculturalEngineer",
            Role::BiologicalEngineer => "BiologicalEngineer",
            Role::DataAnalyst => "DataAnalyst",
            Role::GalacticSecurityHead => "GalacticSecurityHead",
            Role::GalacticSecurityStaff => "GalacticSecurityStaff",
            Role::Mathematician => "Mathematician",
            Role::Physicist => "Physicist",
            Role::Chemist => "Chemist",
            Role::Biologist => "Biologist",
            Role::Astronaut => "Astronaut",
            Role::SettlerCommander => "SettlerCommander",
            Role::CivilEngineer => "CivilEngineer",
            Role::Farmer => "Farmer",
            Role::TemporarySetter => "TemporarySetter",
            Role::SpaceStationSettler => "SpaceStationSettler",
            Role::Psychiatrist => "Psychiatrist",
            Role::PsychiatristAssistant => "PsychiatristAssistant",
            Role::MedicalStaff => "MedicalStaff",
            Role::HeadOfMedicine => "HeadOfMedicine",
            Role::HeadOfSanitary => "HeadOfSanitary",
            Role::InspectorCrew => "InspectorCrew",
            Role::DisposalCrew => "DisposalCrew",
            Role::WastewaterCrew => "WastewaterCrew",
            Role::CleanupCrew => "CleanupCrew",
            Role::TransportCrew => "TransportCrew",
            Role::GeneralDirector => "GeneralDirector",
            Role::TheDirector => "TheDirector",
            Role::TheAccountant => "TheAccountant",
            Role::TheLibrarian => "TheLibrarian",
            Role::TheNomad => "TheNomad",
            Role::TheArtificer => "TheArtificer",
            Role::TheObserver => "TheObserver",
            Role::TheWanderer => "TheWanderer",
            Role::TheTaskmaster => "TheTaskmaster",
            Role::TheGuardian => "TheGuardian",
            Role::TheStatistician => "TheStatistician",
            Role::TheCoordinator => "TheCoordinator",
            Role::TheOverseer => "TheOverseer",
            Role::TheAnchorman => "TheAnchorman",
            Role::Administrator => "Administrator",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The subset of user data held in memory after a successful login.
/// Never includes password_hash. Serialized to Svelte via Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub role: Role,
    pub base_location_id: Option<Uuid>,
    /// Cached from base_locations.has_data_regulation — used by Psychiatry guards
    pub base_has_data_regulation: bool,
}

/// Central application state, managed as a Tauri-managed singleton.
/// Wrapped in Arc<Mutex<>> where mutability is needed.
pub struct AppState {
    pub current_user: Mutex<Option<AuthenticatedUser>>,
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
    pub supabase_storage_url: String,
    /// A valid service_role JWT generated from the SUPABASE_SERVICE_KEY secret
    pub supabase_service_jwt: String,
}
