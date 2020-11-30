use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::num::{ParseFloatError, ParseIntError};
use std::str::{FromStr, ParseBoolError};

/* -- -- -- -- -- -- -- -- -- -- -- -- - FURTHER STRUCTURES - -- -- -- -- -- -- -- -- -- -- -- -- */

pub struct AvailableConfiguration {
    pub name: String,
    pub description: String,
    pub default: String,
    pub data_type: String,
}

#[derive(Debug)]
pub enum SelectError {
    ParseError(String),
}

impl Error for SelectError {}

impl Display for SelectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(error) => {
                write!(f, "ParseError occurred while selecting (\"{}\")", error)
            }
        }
    }
}

impl From<ParseFloatError> for SelectError {
    fn from(error: ParseFloatError) -> Self {
        SelectError::ParseError(format!("{}", error))
    }
}

impl From<ParseBoolError> for SelectError {
    fn from(error: ParseBoolError) -> Self {
        SelectError::ParseError(format!("{}", error))
    }
}

impl From<ParseIntError> for SelectError {
    fn from(error: ParseIntError) -> Self {
        SelectError::ParseError(format!("{}", error))
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- -- -- --   TRAITS   -- -- -- -- -- -- -- -- -- -- -- -- -- -- */

pub trait Available<S: Selected<Self>>: Sized + FromStr {
    fn values() -> Vec<Self>
    where
        Self: std::marker::Sized;
    fn category_headline() -> &'static str;

    fn nice_name(&self) -> &'static str;
    fn long_name(&self) -> &'static str;
    fn short_name(&self) -> &'static str;

    fn available_configurations(&self) -> Vec<AvailableConfiguration>;

    fn select(self, configuration: HashMap<String, String>) -> Result<S, SelectError>;
}

pub trait AvailableSupportsAvailable<S: Selected<A>, A: Available<S>> {
    fn supports_available(&self) -> Vec<A>;
}

pub trait Selected<A: Available<Self>>: Sized + Debug {
    fn corresponding_available(&self) -> A;
}

/* -- -- -- -- -- -- -- -- -- -- -- -- AVAILABLE ENVIRONMENT  -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Clone, PartialEq)]
pub enum AvailableEnvironment {
    GymMountainCar,
    CodeBulletAiLearnsToDrive,
}

impl Available<SelectedEnvironment> for AvailableEnvironment {
    fn values() -> Vec<Self> {
        vec![Self::GymMountainCar, Self::CodeBulletAiLearnsToDrive]
    }

    fn category_headline() -> &'static str {
        "Available Environments"
    }

    fn nice_name(&self) -> &'static str {
        match *self {
            Self::GymMountainCar => "Gym MountainCar",
            Self::CodeBulletAiLearnsToDrive => "Code Bullet AI Learns to DRIVE",
        }
    }

    fn long_name(&self) -> &'static str {
        match *self {
            Self::GymMountainCar => "gym_mountaincar",
            Self::CodeBulletAiLearnsToDrive => "code_bullet_ai_learns_to_drive",
        }
    }

    fn short_name(&self) -> &'static str {
        match *self {
            Self::GymMountainCar => "g_mc",
            Self::CodeBulletAiLearnsToDrive => "cb_drive",
        }
    }

    fn available_configurations(&self) -> Vec<AvailableConfiguration> {
        match *self {
            Self::GymMountainCar => vec![AvailableConfiguration {
                name: "goal_velocity".to_string(),
                description: "The velocity which the agent has to have at least when he reaches \
                the flag. Because the velocity never is negative a value of 0.0 is the off-switch \
                for this."
                    .to_string(),
                default: "0.0".to_string(),
                data_type: "f64".to_string(),
            }],
            Self::CodeBulletAiLearnsToDrive => vec![
                AvailableConfiguration {
                    name: "sensor_lines_visible".to_string(),
                    description: "Whether the given sensor lines should be drawn in the \
                    visualiser. Sometimes it's nice to see what an agent sees."
                        .to_string(),
                    default: "false".to_string(),
                    data_type: "bool".to_string(),
                },
                AvailableConfiguration {
                    name: "track_visible".to_string(),
                    description: "Whether the track should be drawn in the visualiser. This set \
                    to false in addition to \"sensor_lines_visible\" to true simulates the view \
                    the agent has."
                        .to_string(),
                    default: "true".to_string(),
                    data_type: "bool".to_string(),
                },
            ],
        }
    }

    fn select(
        self,
        configuration: HashMap<String, String>,
    ) -> Result<SelectedEnvironment, SelectError> {
        let mut configuration = configuration;
        match self {
            Self::GymMountainCar => Ok(SelectedEnvironment::GymMountainCar {
                goal_velocity: configuration
                    .remove(&"goal_velocity".to_string())
                    .unwrap_or_else(|| "0.0".to_string())
                    .parse::<f64>()?,
            }),
            Self::CodeBulletAiLearnsToDrive => Ok(SelectedEnvironment::CodeBulletAiLearnsToDrive {
                sensor_lines_visible: configuration
                    .remove(&"sensor_lines_visible".to_string())
                    .unwrap_or_else(|| "false".to_string())
                    .parse::<bool>()?,
                track_visible: configuration
                    .remove(&"track_visible".to_string())
                    .unwrap_or_else(|| "true".to_string())
                    .parse::<bool>()?,
            }),
        }
    }
}

impl FromStr for AvailableEnvironment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s = s.to_lowercase();
        Self::values()
            .into_iter()
            .find(|element| {
                element.nice_name().to_lowercase().eq(&lower_s)
                    || element.long_name().to_lowercase().eq(&lower_s)
                    || element.short_name().to_lowercase().eq(&lower_s)
            })
            .ok_or_else(|| format!("Did not find \"{}\" in available environments.", lower_s))
    }
}

impl AvailableSupportsAvailable<SelectedAgent, AvailableAgent> for AvailableEnvironment {
    fn supports_available(&self) -> Vec<AvailableAgent> {
        match *self {
            Self::GymMountainCar => vec![AvailableAgent::Input, AvailableAgent::Random],
            Self::CodeBulletAiLearnsToDrive => vec![AvailableAgent::Input, AvailableAgent::Random],
        }
    }
}

impl AvailableSupportsAvailable<SelectedVisualiser, AvailableVisualiser> for AvailableEnvironment {
    fn supports_available(&self) -> Vec<AvailableVisualiser> {
        match *self {
            Self::GymMountainCar => {
                vec![AvailableVisualiser::None, AvailableVisualiser::PistonIn2d]
            }
            Self::CodeBulletAiLearnsToDrive => {
                vec![AvailableVisualiser::None, AvailableVisualiser::PistonIn2d]
            }
        }
    }
}

impl AvailableSupportsAvailable<SelectedExitCondition, AvailableExitCondition>
    for AvailableEnvironment
{
    fn supports_available(&self) -> Vec<AvailableExitCondition> {
        match *self {
            Self::GymMountainCar => vec![
                AvailableExitCondition::EpisodesSimulated,
                AvailableExitCondition::VisualiserClosed,
            ],
            Self::CodeBulletAiLearnsToDrive => vec![
                AvailableExitCondition::EpisodesSimulated,
                AvailableExitCondition::VisualiserClosed,
            ],
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- --  SELECTED ENVIRONMENT  -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Debug)]
pub enum SelectedEnvironment {
    GymMountainCar {
        goal_velocity: f64,
    },
    CodeBulletAiLearnsToDrive {
        sensor_lines_visible: bool,
        track_visible: bool,
    },
}

impl Selected<AvailableEnvironment> for SelectedEnvironment {
    fn corresponding_available(&self) -> AvailableEnvironment {
        match *self {
            Self::GymMountainCar { .. } => AvailableEnvironment::GymMountainCar,
            Self::CodeBulletAiLearnsToDrive { .. } => {
                AvailableEnvironment::CodeBulletAiLearnsToDrive
            }
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- -- -- AVAILABLE AGENT  -- -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Clone, PartialEq)]
pub enum AvailableAgent {
    Random,
    Input,
}

impl Available<SelectedAgent> for AvailableAgent {
    fn values() -> Vec<Self> {
        vec![Self::Random, Self::Input]
    }

    fn category_headline() -> &'static str {
        "Available Agents"
    }

    fn nice_name(&self) -> &'static str {
        match *self {
            Self::Random => "Random",
            Self::Input => "Input",
        }
    }

    fn long_name(&self) -> &'static str {
        match *self {
            Self::Random => "random",
            Self::Input => "input",
        }
    }

    fn short_name(&self) -> &'static str {
        match *self {
            Self::Random => "rand",
            Self::Input => "inp",
        }
    }

    fn available_configurations(&self) -> Vec<AvailableConfiguration> {
        match *self {
            Self::Random => vec![],
            Self::Input => vec![],
        }
    }

    fn select(self, _configuration: HashMap<String, String>) -> Result<SelectedAgent, SelectError> {
        match self {
            Self::Random => Ok(SelectedAgent::Random),
            Self::Input => Ok(SelectedAgent::Input),
        }
    }
}

impl FromStr for AvailableAgent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s = s.to_lowercase();
        Self::values()
            .into_iter()
            .find(|element| {
                element.nice_name().to_lowercase().eq(&lower_s)
                    || element.long_name().to_lowercase().eq(&lower_s)
                    || element.short_name().to_lowercase().eq(&lower_s)
            })
            .ok_or_else(|| format!("Did not find \"{}\" in available agents.", lower_s))
    }
}

impl AvailableSupportsAvailable<SelectedEnvironment, AvailableEnvironment> for AvailableAgent {
    fn supports_available(&self) -> Vec<AvailableEnvironment> {
        match *self {
            Self::Random => vec![
                AvailableEnvironment::GymMountainCar,
                AvailableEnvironment::CodeBulletAiLearnsToDrive,
            ],
            Self::Input => vec![
                AvailableEnvironment::GymMountainCar,
                AvailableEnvironment::CodeBulletAiLearnsToDrive,
            ],
        }
    }
}

impl AvailableSupportsAvailable<SelectedVisualiser, AvailableVisualiser> for AvailableAgent {
    fn supports_available(&self) -> Vec<AvailableVisualiser> {
        match *self {
            Self::Random => vec![AvailableVisualiser::None, AvailableVisualiser::PistonIn2d],
            Self::Input => vec![AvailableVisualiser::None, AvailableVisualiser::PistonIn2d],
        }
    }
}

impl AvailableSupportsAvailable<SelectedExitCondition, AvailableExitCondition> for AvailableAgent {
    fn supports_available(&self) -> Vec<AvailableExitCondition> {
        match *self {
            Self::Random => vec![
                AvailableExitCondition::EpisodesSimulated,
                AvailableExitCondition::VisualiserClosed,
            ],
            Self::Input => vec![
                AvailableExitCondition::EpisodesSimulated,
                AvailableExitCondition::VisualiserClosed,
            ],
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- -- --  SELECTED AGENT  -- -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Debug)]
pub enum SelectedAgent {
    Random,
    Input,
}

impl Selected<AvailableAgent> for SelectedAgent {
    fn corresponding_available(&self) -> AvailableAgent {
        match *self {
            Self::Random => AvailableAgent::Random,
            Self::Input => AvailableAgent::Input,
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- -- AVAILABLE VISUALISER   -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Clone, PartialEq)]
pub enum AvailableVisualiser {
    None,
    PistonIn2d,
}

impl Available<SelectedVisualiser> for AvailableVisualiser {
    fn values() -> Vec<Self> {
        vec![Self::None, Self::PistonIn2d]
    }

    fn category_headline() -> &'static str {
        "Available Visualisers"
    }

    fn nice_name(&self) -> &'static str {
        match *self {
            Self::None => "None",
            Self::PistonIn2d => "Piston in 2D",
        }
    }

    fn long_name(&self) -> &'static str {
        match *self {
            Self::None => "none",
            Self::PistonIn2d => "piston2d",
        }
    }

    fn short_name(&self) -> &'static str {
        match *self {
            Self::None => "none",
            Self::PistonIn2d => "pi2d",
        }
    }

    fn available_configurations(&self) -> Vec<AvailableConfiguration> {
        match *self {
            Self::None => vec![],
            Self::PistonIn2d => vec![
                AvailableConfiguration {
                    name: "window_title".to_string(),
                    description: "Sets the window title.".to_string(),
                    default: "Gymnarium Application".to_string(),
                    data_type: "String".to_string(),
                },
                AvailableConfiguration {
                    name: "window_dimension".to_string(),
                    description: "Sets the window dimensions with which it should start. It's \
                    important to specify them with the parentheses and the comma."
                        .to_string(),
                    default: "(640, 480)".to_string(),
                    data_type: "(u32, u32)".to_string(),
                },
            ],
        }
    }

    fn select(
        self,
        configuration: HashMap<String, String>,
    ) -> Result<SelectedVisualiser, SelectError> {
        fn tuple_u32_u32_from_str(s: &str) -> Result<(u32, u32), String> {
            let numbers = if s.starts_with('(') && s.ends_with(')') {
                &s[1..s.len() - 1]
            } else {
                &s
            }
            .split(',')
            .map(|number_string| number_string.trim().parse::<u32>())
            .collect::<Result<Vec<u32>, ParseIntError>>()
            .map_err(|error| format!("{}", error))?;
            Ok((numbers[0], numbers[1]))
        }

        let mut configuration = configuration;
        match self {
            Self::None => Ok(SelectedVisualiser::None),
            Self::PistonIn2d => Ok(SelectedVisualiser::PistonIn2d {
                window_title: configuration
                    .remove(&"window_title".to_string())
                    .unwrap_or_else(|| "Gymnarium Application".to_string()),
                window_dimension: configuration
                    .remove(&"window_dimension".to_string())
                    .and_then(|value| tuple_u32_u32_from_str(&value).ok())
                    .or(Some((640, 480)))
                    .unwrap(),
            }),
        }
    }
}

impl FromStr for AvailableVisualiser {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s = s.to_lowercase();
        Self::values()
            .into_iter()
            .find(|element| {
                element.nice_name().to_lowercase().eq(&lower_s)
                    || element.long_name().to_lowercase().eq(&lower_s)
                    || element.short_name().to_lowercase().eq(&lower_s)
            })
            .ok_or_else(|| format!("Did not find \"{}\" in available visualisers.", lower_s))
    }
}

impl AvailableSupportsAvailable<SelectedEnvironment, AvailableEnvironment> for AvailableVisualiser {
    fn supports_available(&self) -> Vec<AvailableEnvironment> {
        match *self {
            Self::None => vec![
                AvailableEnvironment::GymMountainCar,
                AvailableEnvironment::CodeBulletAiLearnsToDrive,
            ],
            Self::PistonIn2d => vec![
                AvailableEnvironment::GymMountainCar,
                AvailableEnvironment::CodeBulletAiLearnsToDrive,
            ],
        }
    }
}

impl AvailableSupportsAvailable<SelectedAgent, AvailableAgent> for AvailableVisualiser {
    fn supports_available(&self) -> Vec<AvailableAgent> {
        match *self {
            Self::None => vec![AvailableAgent::Random, AvailableAgent::Input],
            Self::PistonIn2d => vec![AvailableAgent::Random, AvailableAgent::Input],
        }
    }
}

impl AvailableSupportsAvailable<SelectedExitCondition, AvailableExitCondition>
    for AvailableVisualiser
{
    fn supports_available(&self) -> Vec<AvailableExitCondition> {
        match *self {
            Self::None => vec![
                AvailableExitCondition::EpisodesSimulated,
                AvailableExitCondition::VisualiserClosed,
            ],
            Self::PistonIn2d => vec![
                AvailableExitCondition::EpisodesSimulated,
                AvailableExitCondition::VisualiserClosed,
            ],
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- --  SELECTED VISUALISER   -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Debug)]
pub enum SelectedVisualiser {
    None,
    PistonIn2d {
        window_title: String,
        window_dimension: (u32, u32),
    },
}

impl Selected<AvailableVisualiser> for SelectedVisualiser {
    fn corresponding_available(&self) -> AvailableVisualiser {
        match *self {
            Self::None => AvailableVisualiser::None,
            Self::PistonIn2d { .. } => AvailableVisualiser::PistonIn2d,
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- --  AVAILABLE EXIT CONDITION -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Clone, PartialEq)]
pub enum AvailableExitCondition {
    EpisodesSimulated,
    VisualiserClosed,
}

impl Available<SelectedExitCondition> for AvailableExitCondition {
    fn values() -> Vec<Self> {
        vec![Self::EpisodesSimulated, Self::VisualiserClosed]
    }

    fn category_headline() -> &'static str {
        "Available Exit Conditions"
    }

    fn nice_name(&self) -> &'static str {
        match *self {
            Self::EpisodesSimulated => "episodes done simulating",
            Self::VisualiserClosed => "visualiser is closed",
        }
    }

    fn long_name(&self) -> &'static str {
        match *self {
            Self::EpisodesSimulated => "episodes_done_simulating",
            Self::VisualiserClosed => "visualiser_is_closed",
        }
    }

    fn short_name(&self) -> &'static str {
        match *self {
            Self::EpisodesSimulated => "epsdone",
            Self::VisualiserClosed => "visclosed",
        }
    }

    fn available_configurations(&self) -> Vec<AvailableConfiguration> {
        match *self {
            Self::EpisodesSimulated => vec![AvailableConfiguration {
                name: "count_of_episodes".to_string(),
                description: "The number of episodes to run through before exiting.".to_string(),
                default: "20".to_string(),
                data_type: "u128".to_string(),
            }],
            Self::VisualiserClosed => vec![],
        }
    }

    fn select(
        self,
        configuration: HashMap<String, String>,
    ) -> Result<SelectedExitCondition, SelectError> {
        let mut configuration = configuration;
        match self {
            Self::EpisodesSimulated => Ok(SelectedExitCondition::EpisodesSimulated {
                count_of_episodes: configuration
                    .remove(&"count_of_episodes".to_string())
                    .unwrap_or_else(|| "20".to_string())
                    .parse::<u128>()?,
            }),
            Self::VisualiserClosed => Ok(SelectedExitCondition::VisualiserClosed),
        }
    }
}

impl FromStr for AvailableExitCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s = s.to_lowercase();
        Self::values()
            .into_iter()
            .find(|element| {
                element.nice_name().to_lowercase().eq(&lower_s)
                    || element.long_name().to_lowercase().eq(&lower_s)
                    || element.short_name().to_lowercase().eq(&lower_s)
            })
            .ok_or_else(|| format!("Did not find \"{}\" in available exit conditions.", lower_s))
    }
}

impl AvailableSupportsAvailable<SelectedEnvironment, AvailableEnvironment>
    for AvailableExitCondition
{
    fn supports_available(&self) -> Vec<AvailableEnvironment> {
        match *self {
            Self::EpisodesSimulated => vec![
                AvailableEnvironment::GymMountainCar,
                AvailableEnvironment::CodeBulletAiLearnsToDrive,
            ],
            Self::VisualiserClosed => vec![
                AvailableEnvironment::GymMountainCar,
                AvailableEnvironment::CodeBulletAiLearnsToDrive,
            ],
        }
    }
}

impl AvailableSupportsAvailable<SelectedAgent, AvailableAgent> for AvailableExitCondition {
    fn supports_available(&self) -> Vec<AvailableAgent> {
        match *self {
            Self::EpisodesSimulated => vec![AvailableAgent::Random, AvailableAgent::Input],
            Self::VisualiserClosed => vec![AvailableAgent::Random, AvailableAgent::Input],
        }
    }
}

impl AvailableSupportsAvailable<SelectedVisualiser, AvailableVisualiser>
    for AvailableExitCondition
{
    fn supports_available(&self) -> Vec<AvailableVisualiser> {
        match *self {
            Self::EpisodesSimulated => {
                vec![AvailableVisualiser::None, AvailableVisualiser::PistonIn2d]
            }
            Self::VisualiserClosed => {
                vec![AvailableVisualiser::None, AvailableVisualiser::PistonIn2d]
            }
        }
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- - SELECTED EXIT CONDITION -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Debug)]
pub enum SelectedExitCondition {
    EpisodesSimulated { count_of_episodes: u128 },
    VisualiserClosed,
}

impl Selected<AvailableExitCondition> for SelectedExitCondition {
    fn corresponding_available(&self) -> AvailableExitCondition {
        match *self {
            Self::EpisodesSimulated { .. } => AvailableExitCondition::EpisodesSimulated,
            Self::VisualiserClosed => AvailableExitCondition::VisualiserClosed,
        }
    }
}

/*  -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --  */
