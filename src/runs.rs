use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::read_to_string;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use gymnarium::gymnarium_base::{Agent, Environment, Seed};
use gymnarium::gymnarium_visualisers_base::{
    DrawableEnvironment, PixelArrayDrawableEnvironment, PixelArrayVisualiser,
    TextDrawableEnvironment, TextVisualiser, ThreeDimensionalDrawableEnvironment,
    ThreeDimensionalVisualiser, TwoDimensionalDrawableEnvironment, TwoDimensionalVisualiser,
    Visualiser,
};

/* -- -- -- -- -- -- -- -- -- -- -- -- - FURTHER STRUCTURES - -- -- -- -- -- -- -- -- -- -- -- -- */

pub struct RunOptions {
    pub seed: Option<Seed>,
    pub reset_environment_on_done: bool,
    pub reset_agent_on_done: bool,
    pub environment_load_path: Option<String>,
    pub environment_store_path: Option<String>,
    pub agent_load_path: Option<String>,
    pub agent_store_path: Option<String>,
}

/* -- -- -- -- -- -- -- -- -- -- -- -- --  NO VISUALISER   -- -- -- -- -- -- -- -- -- -- -- -- -- */

pub fn run_with_no_visualiser<
    EError: Error,
    EInfo: Debug,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData>,
    AError: Error,
    AData: Serialize + DeserializeOwned,
    A: Agent<AError, AData>,
    XCF: Fn(&E, &A, u128, u128) -> bool,
>(
    environment: E,
    agent: A,
    exit_condition: XCF,
    run_options: RunOptions,
) {
    let mut environment = environment;
    let mut agent = agent;

    let mut state = if let Some(environment_load_path_string) = run_options.environment_load_path {
        load_environment(&mut environment, environment_load_path_string).unwrap();
        environment.state()
    } else {
        environment.reseed(run_options.seed.clone()).unwrap();
        environment.reset().unwrap()
    };

    if let Some(agent_load_path_string) = run_options.agent_load_path {
        load_agent(&mut agent, agent_load_path_string).unwrap();
    } else {
        agent.reseed(run_options.seed).unwrap();
        agent.reset().unwrap();
    }

    let mut episode = 0u128;
    let mut step = 0u128;

    while !exit_condition(&environment, &agent, episode, step) {
        let action = agent.choose_action(&state).unwrap();

        let (new_state, reward, done, _) = environment.step(&action).unwrap();
        step += 1;

        agent
            .process_reward(&state, &new_state, reward, done)
            .unwrap();

        state = if run_options.reset_environment_on_done && done {
            step = 0;
            episode += 1;
            environment.reset().unwrap()
        } else {
            new_state
        };

        if run_options.reset_agent_on_done && done {
            agent.reset().unwrap();
        }
    }

    if let Some(agent_store_path_string) = run_options.agent_store_path {
        store_agent(&agent, agent_store_path_string).unwrap();
    }

    if let Some(environment_store_path_string) = run_options.environment_store_path {
        store_environment(&environment, environment_store_path_string).unwrap();
    }

    agent.close().unwrap();
    environment.close().unwrap();
}

/* -- -- -- -- -- -- -- -- -- -- --  TWO DIMENSIONAL VISUALISER  -- -- -- -- -- -- -- -- -- -- -- */

pub fn run_with_two_dimensional_visualiser<
    EError: Error,
    EInfo: Debug,
    DEError: Error,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData>
        + DrawableEnvironment
        + TwoDimensionalDrawableEnvironment<DEError>,
    AError: Error,
    AData: Serialize + DeserializeOwned,
    A: Agent<AError, AData>,
    VError: Error,
    TDVError: Error,
    V: Visualiser<VError> + TwoDimensionalVisualiser<TDVError, VError, DEError>,
    XCF: Fn(&E, &A, &V, u128, u128) -> bool,
>(
    environment: E,
    agent: A,
    visualiser: V,
    exit_condition: XCF,
    run_options: RunOptions,
) {
    let mut environment = environment;
    let mut agent = agent;
    let mut visualiser = visualiser;

    let mut state = if let Some(environment_load_path_string) = run_options.environment_load_path {
        load_environment(&mut environment, environment_load_path_string).unwrap();
        environment.state()
    } else {
        environment.reseed(run_options.seed.clone()).unwrap();
        environment.reset().unwrap()
    };

    visualiser.render_two_dimensional(&environment).unwrap();

    if let Some(agent_load_path_string) = run_options.agent_load_path {
        load_agent(&mut agent, agent_load_path_string).unwrap();
    } else {
        agent.reseed(run_options.seed).unwrap();
        agent.reset().unwrap();
    }

    let mut episode = 0u128;
    let mut step = 0u128;

    while !exit_condition(&environment, &agent, &visualiser, episode, step) {
        let action = agent.choose_action(&state).unwrap();

        let (new_state, reward, done, _) = environment.step(&action).unwrap();
        step += 1;

        agent
            .process_reward(&state, &new_state, reward, done)
            .unwrap();

        state = if run_options.reset_environment_on_done && done {
            step = 0;
            episode += 1;
            environment.reset().unwrap()
        } else {
            new_state
        };

        if run_options.reset_agent_on_done && done {
            agent.reset().unwrap();
        }

        visualiser.render_two_dimensional(&environment).unwrap();

        sleep_suggested_steps_per_second_or_30_fps::<E>();
    }

    if let Some(agent_store_path_string) = run_options.agent_store_path {
        store_agent(&agent, agent_store_path_string).unwrap();
    }

    if let Some(environment_store_path_string) = run_options.environment_store_path {
        store_environment(&environment, environment_store_path_string).unwrap();
    }

    agent.close().unwrap();
    environment.close().unwrap();
    visualiser.close().unwrap();
}

/* -- -- -- -- -- -- -- -- -- -- -- THREE DIMENSIONAL VISUALISER -- -- -- -- -- -- -- -- -- -- -- */

pub fn _run_with_three_dimensional_visualiser<
    EError: Error,
    EInfo: Debug,
    DEError: Error,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData>
        + DrawableEnvironment
        + ThreeDimensionalDrawableEnvironment<DEError>,
    AError: Error,
    AData: Serialize + DeserializeOwned,
    A: Agent<AError, AData>,
    VError: Error,
    TDVError: Error,
    V: Visualiser<VError> + ThreeDimensionalVisualiser<TDVError, VError, DEError>,
    XCF: Fn(&E, &A, &V, u128, u128) -> bool,
>(
    environment: E,
    agent: A,
    visualiser: V,
    exit_condition: XCF,
    run_options: RunOptions,
) {
    let mut environment = environment;
    let mut agent = agent;
    let mut visualiser = visualiser;

    let mut state = if let Some(environment_load_path_string) = run_options.environment_load_path {
        load_environment(&mut environment, environment_load_path_string).unwrap();
        environment.state()
    } else {
        environment.reseed(run_options.seed.clone()).unwrap();
        environment.reset().unwrap()
    };

    visualiser.render_three_dimensional(&environment).unwrap();

    if let Some(agent_load_path_string) = run_options.agent_load_path {
        load_agent(&mut agent, agent_load_path_string).unwrap();
    } else {
        agent.reseed(run_options.seed).unwrap();
        agent.reset().unwrap();
    }

    let mut episode = 0u128;
    let mut step = 0u128;

    while !exit_condition(&environment, &agent, &visualiser, episode, step) {
        let action = agent.choose_action(&state).unwrap();

        let (new_state, reward, done, _) = environment.step(&action).unwrap();
        step += 1;

        agent
            .process_reward(&state, &new_state, reward, done)
            .unwrap();

        state = if run_options.reset_environment_on_done && done {
            step = 0;
            episode += 1;
            environment.reset().unwrap()
        } else {
            new_state
        };

        if run_options.reset_agent_on_done && done {
            agent.reset().unwrap();
        }

        visualiser.render_three_dimensional(&environment).unwrap();

        sleep_suggested_steps_per_second_or_30_fps::<E>();
    }

    if let Some(agent_store_path_string) = run_options.agent_store_path {
        store_agent(&agent, agent_store_path_string).unwrap();
    }

    if let Some(environment_store_path_string) = run_options.environment_store_path {
        store_environment(&environment, environment_store_path_string).unwrap();
    }

    agent.close().unwrap();
    environment.close().unwrap();
    visualiser.close().unwrap();
}

/* -- -- -- -- -- -- -- -- -- -- -- -- PIXEL ARRAY VISUALISER -- -- -- -- -- -- -- -- -- -- -- -- */

pub fn _run_with_pixel_array_visualiser<
    EError: Error,
    EInfo: Debug,
    DEError: Error,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData>
        + DrawableEnvironment
        + PixelArrayDrawableEnvironment<DEError>,
    AError: Error,
    AData: Serialize + DeserializeOwned,
    A: Agent<AError, AData>,
    VError: Error,
    TDVError: Error,
    V: Visualiser<VError> + PixelArrayVisualiser<TDVError, VError, DEError>,
    XCF: Fn(&E, &A, &V, u128, u128) -> bool,
>(
    environment: E,
    agent: A,
    visualiser: V,
    exit_condition: XCF,
    run_options: RunOptions,
) {
    let mut environment = environment;
    let mut agent = agent;
    let mut visualiser = visualiser;

    let mut state = if let Some(environment_load_path_string) = run_options.environment_load_path {
        load_environment(&mut environment, environment_load_path_string).unwrap();
        environment.state()
    } else {
        environment.reseed(run_options.seed.clone()).unwrap();
        environment.reset().unwrap()
    };

    visualiser.render_pixel_array(&environment).unwrap();

    if let Some(agent_load_path_string) = run_options.agent_load_path {
        load_agent(&mut agent, agent_load_path_string).unwrap();
    } else {
        agent.reseed(run_options.seed).unwrap();
        agent.reset().unwrap();
    }

    let mut episode = 0u128;
    let mut step = 0u128;

    while !exit_condition(&environment, &agent, &visualiser, episode, step) {
        let action = agent.choose_action(&state).unwrap();

        let (new_state, reward, done, _) = environment.step(&action).unwrap();
        step += 1;

        agent
            .process_reward(&state, &new_state, reward, done)
            .unwrap();

        state = if run_options.reset_environment_on_done && done {
            step = 0;
            episode += 1;
            environment.reset().unwrap()
        } else {
            new_state
        };

        if run_options.reset_agent_on_done && done {
            agent.reset().unwrap();
        }

        visualiser.render_pixel_array(&environment).unwrap();

        sleep_suggested_steps_per_second_or_30_fps::<E>();
    }

    if let Some(agent_store_path_string) = run_options.agent_store_path {
        store_agent(&agent, agent_store_path_string).unwrap();
    }

    if let Some(environment_store_path_string) = run_options.environment_store_path {
        store_environment(&environment, environment_store_path_string).unwrap();
    }

    agent.close().unwrap();
    environment.close().unwrap();
    visualiser.close().unwrap();
}

/* -- -- -- -- -- -- -- -- -- -- -- -- -- TEXT VISUALISER  -- -- -- -- -- -- -- -- -- -- -- -- -- */

pub fn _run_with_text_visualiser<
    EError: Error,
    EInfo: Debug,
    DEError: Error,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData> + DrawableEnvironment + TextDrawableEnvironment<DEError>,
    AError: Error,
    AData: Serialize + DeserializeOwned,
    A: Agent<AError, AData>,
    VError: Error,
    TDVError: Error,
    V: Visualiser<VError> + TextVisualiser<TDVError, VError, DEError>,
    XCF: Fn(&E, &A, &V, u128, u128) -> bool,
>(
    environment: E,
    agent: A,
    visualiser: V,
    exit_condition: XCF,
    run_options: RunOptions,
) {
    let mut environment = environment;
    let mut agent = agent;
    let mut visualiser = visualiser;

    let mut state = if let Some(environment_load_path_string) = run_options.environment_load_path {
        load_environment(&mut environment, environment_load_path_string).unwrap();
        environment.state()
    } else {
        environment.reseed(run_options.seed.clone()).unwrap();
        environment.reset().unwrap()
    };

    visualiser.render_text(&environment).unwrap();

    if let Some(agent_load_path_string) = run_options.agent_load_path {
        load_agent(&mut agent, agent_load_path_string).unwrap();
    } else {
        agent.reseed(run_options.seed).unwrap();
        agent.reset().unwrap();
    }

    let mut episode = 0u128;
    let mut step = 0u128;

    while !exit_condition(&environment, &agent, &visualiser, episode, step) {
        let action = agent.choose_action(&state).unwrap();

        let (new_state, reward, done, _) = environment.step(&action).unwrap();
        step += 1;

        agent
            .process_reward(&state, &new_state, reward, done)
            .unwrap();

        state = if run_options.reset_environment_on_done && done {
            step = 0;
            episode += 1;
            environment.reset().unwrap()
        } else {
            new_state
        };

        if run_options.reset_agent_on_done && done {
            agent.reset().unwrap();
        }

        visualiser.render_text(&environment).unwrap();

        sleep_suggested_steps_per_second_or_30_fps::<E>();
    }

    if let Some(agent_store_path_string) = run_options.agent_store_path {
        store_agent(&agent, agent_store_path_string).unwrap();
    }

    if let Some(environment_store_path_string) = run_options.environment_store_path {
        store_environment(&environment, environment_store_path_string).unwrap();
    }

    agent.close().unwrap();
    environment.close().unwrap();
    visualiser.close().unwrap();
}

/* -- -- -- -- -- -- -- -- -- -- -- -- -- -- - HELPER - -- -- -- -- -- -- -- -- -- -- -- -- -- -- */

pub fn sleep_suggested_steps_per_second_or_30_fps<E: DrawableEnvironment>() {
    if let Some(rsps) = E::suggested_rendered_steps_per_second() {
        std::thread::sleep(Duration::from_millis((1000f64 / rsps) as u64));
    } else {
        std::thread::sleep(Duration::from_millis((1000f64 / 30f64) as u64));
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- -- --  LOAD AND STORE  -- -- -- -- -- -- -- -- -- -- -- -- -- */

#[derive(Debug)]
enum LoadError<EAError: Error> {
    IoError(std::io::Error),
    SerdeJsonError(serde_json::Error),
    RonError(ron::error::Error),
    EnvironmentAgentError(EAError),
    UnknownFormat(String),
}

impl<EAError: Error> Display for LoadError<EAError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(error) => write!(f, "Received IoError ({})", error),
            Self::SerdeJsonError(error) => write!(f, "Received SerdeJsonError ({})", error),
            Self::RonError(error) => write!(f, "Received RonError ({})", error),
            Self::EnvironmentAgentError(error) => {
                write!(f, "Recedived EnvironmentError({})", error)
            }
            Self::UnknownFormat(path) => {
                write!(f, "The file \"{}\" has an unknown file ending", path)
            }
        }
    }
}

impl<EAError: Error> Error for LoadError<EAError> {}

impl<EAError: Error> From<std::io::Error> for LoadError<EAError> {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl<EAError: Error> From<serde_json::error::Error> for LoadError<EAError> {
    fn from(error: serde_json::error::Error) -> Self {
        Self::SerdeJsonError(error)
    }
}

impl<EAError: Error> From<ron::error::Error> for LoadError<EAError> {
    fn from(error: ron::error::Error) -> Self {
        Self::RonError(error)
    }
}

fn load_environment<
    EError: Error,
    EInfo: Debug,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData>,
>(
    environment: &mut E,
    environment_load_path_string: String,
) -> Result<(), LoadError<EError>> {
    if environment_load_path_string.ends_with(".json") {
        let file_contents = read_to_string(environment_load_path_string)?;
        let deserialized = serde_json::from_str(&file_contents)?;
        environment
            .load(deserialized)
            .map_err(LoadError::EnvironmentAgentError)?;
        Ok(())
    } else if environment_load_path_string.ends_with(".ron") {
        let file_contents = read_to_string(environment_load_path_string)?;
        let deserialized = ron::from_str(&file_contents)?;
        environment
            .load(deserialized)
            .map_err(LoadError::EnvironmentAgentError)?;
        Ok(())
    } else {
        Err(LoadError::UnknownFormat(environment_load_path_string))
    }
}

fn load_agent<AError: Error, AData: Serialize + DeserializeOwned, A: Agent<AError, AData>>(
    agent: &mut A,
    agent_load_path_string: String,
) -> Result<(), LoadError<AError>> {
    if agent_load_path_string.ends_with(".json") {
        let file_contents = read_to_string(agent_load_path_string)?;
        let deserialized = serde_json::from_str(&file_contents)?;
        agent
            .load(deserialized)
            .map_err(LoadError::EnvironmentAgentError)?;
        Ok(())
    } else if agent_load_path_string.ends_with(".ron") {
        let file_contents = read_to_string(agent_load_path_string)?;
        let deserialized = ron::from_str(&file_contents)?;
        agent
            .load(deserialized)
            .map_err(LoadError::EnvironmentAgentError)?;
        Ok(())
    } else {
        Err(LoadError::UnknownFormat(agent_load_path_string))
    }
}

#[derive(Debug)]
enum StoreError {
    IoError(std::io::Error),
    SerdeJsonError(serde_json::Error),
    RonError(ron::error::Error),
    UnknownFormat(String),
}

impl Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(error) => write!(f, "Received IoError ({})", error),
            Self::SerdeJsonError(error) => write!(f, "Received SerdeJsonError ({})", error),
            Self::RonError(error) => write!(f, "Received RonError ({})", error),
            Self::UnknownFormat(path) => {
                write!(f, "The file \"{}\" has an unknown file ending", path)
            }
        }
    }
}

impl Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<serde_json::error::Error> for StoreError {
    fn from(error: serde_json::error::Error) -> Self {
        Self::SerdeJsonError(error)
    }
}

impl From<ron::error::Error> for StoreError {
    fn from(error: ron::error::Error) -> Self {
        Self::RonError(error)
    }
}

fn store_environment<
    EError: Error,
    EInfo: Debug,
    EData: Serialize + DeserializeOwned,
    E: Environment<EError, EInfo, EData>,
>(
    environment: &E,
    environment_store_path_string: String,
) -> Result<(), StoreError> {
    if environment_store_path_string.ends_with(".json") {
        let data = environment.store();
        let serialized = serde_json::to_string(&data)?;
        std::fs::write(environment_store_path_string, serialized)?;
        Ok(())
    } else if environment_store_path_string.ends_with(".ron") {
        let data = environment.store();
        let serialized = ron::to_string(&data)?;
        std::fs::write(environment_store_path_string, serialized)?;
        Ok(())
    } else {
        Err(StoreError::UnknownFormat(environment_store_path_string))
    }
}

fn store_agent<AError: Error, AData: Serialize + DeserializeOwned, A: Agent<AError, AData>>(
    agent: &A,
    agent_store_path_string: String,
) -> Result<(), StoreError> {
    if agent_store_path_string.ends_with(".json") {
        let data = agent.store();
        let serialized = serde_json::to_string(&data)?;
        std::fs::write(agent_store_path_string, serialized)?;
        Ok(())
    } else if agent_store_path_string.ends_with(".ron") {
        let data = agent.store();
        let serialized = ron::to_string(&data)?;
        std::fs::write(agent_store_path_string, serialized)?;
        Ok(())
    } else {
        Err(StoreError::UnknownFormat(agent_store_path_string))
    }
}

/* -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- ---- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- */
