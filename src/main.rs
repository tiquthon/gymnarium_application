extern crate clap;
extern crate gymnarium;
extern crate ron;
extern crate serde;
extern crate serde_json;

mod availables;
mod runs;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::io::Write;
use std::str::FromStr;

use clap::{
    crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};

use serde::de::DeserializeOwned;
use serde::Serialize;

use gymnarium::gymnarium_agents_random::RandomAgent;
use gymnarium::gymnarium_base::{ActionSpace, Agent, Environment, Seed, ToActionMapper};
use gymnarium::gymnarium_environments_gym::mountain_car::{
    MountainCar, MountainCarInputToActionMapper,
};
use gymnarium::gymnarium_environments_tiquthon::code_bullet::ai_learns_to_drive::{
    AiLearnsToDrive, AiLearnsToDriveInputToActionMapper,
};
use gymnarium::gymnarium_visualisers_base::{
    input, DrawableEnvironment, InputAgent, InputProvider, TwoDimensionalDrawableEnvironment,
    TwoDimensionalVisualiser, Visualiser,
};
use gymnarium::gymnarium_visualisers_piston::PistonVisualiser;

use crate::availables::*;
use crate::runs::{run_with_no_visualiser, run_with_two_dimensional_visualiser, RunOptions};

const APP_NAME: &str = "Gymnarium Application";

fn main() {
    fn format_configuration_options<S: Selected<A>, A: Available<S>>(available: A) -> String {
        let available_configurations = available.available_configurations();
        format!(
            "- {}: {}",
            available.nice_name(),
            if available_configurations.is_empty() {
                "n/a\r\n".to_string()
            } else {
                format!(
                    "{}\r\n",
                    available_configurations
                        .into_iter()
                        .map(|available_configuration| format!(
                            "\r\n  > {} [{}; default: {}]\r\n    {}",
                            available_configuration.name,
                            available_configuration.data_type,
                            available_configuration.default,
                            available_configuration.description
                        ))
                        .fold(String::new(), |result, line| result + &line)
                )
            }
        )
    }

    fn format_available_value<S: Selected<A>, A: Available<S>>(available: A) -> String {
        format!(
            "  \r\n- {} ({}, {})",
            available.nice_name(),
            available.long_name(),
            available.short_name()
        )
    }

    let matches = App::new(APP_NAME)
        .version(crate_version!())
        .author(crate_authors!(", "))
        .about(crate_description!())
        .long_about("")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(SubCommand::with_name("interactive")
            .about("asks every configurable option interactively"))
        .subcommand(SubCommand::with_name("command_line")
            .about("only accepts command line arguments; see `command_line --help` for help")
            .arg(Arg::with_name("environment")
                .short("e")
                .long("environment")
                .help("specifies the environment to simulate")
                .long_help(&format!(
                    "Specifies the environment which should be simulated. There are limited \
                environments baked into this application. Each environment has its own \
                configuration. See `--environment-configuration` for this.\r\n\r\nCurrently there \
                are {} environments baked into this application:{}\r\n",
                    AvailableEnvironment::values().len(),
                    AvailableEnvironment::values()
                        .into_iter()
                        .map(format_available_value)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .required(true)
                .takes_value(true)
                .hide_possible_values(true)
                .possible_values(
                    &AvailableEnvironment::values()
                        .into_iter()
                        .map(|e| vec![
                            e.nice_name(), e.short_name(), e.long_name()
                        ].into_iter())
                        .flatten()
                        .collect::<Vec<&str>>()
                )
                .case_insensitive(true)
                .value_name("ENVIRONMENT")
                .display_order(10)
            )
            .arg(Arg::with_name("environment_configuration")
                .short("f")
                .long("environment-configuration")
                .help("configures the specified environment")
                .long_help(&format!(
                    "Configures the specified environment. The configuration is formatted as \"key=\
                    value;key=value;key=value\" while all additional non formating ';' and '\\' \
                    are escaped with '\\' like \"key=val\\;ue;ke\\;y=va\\\\lue\".\r\n\r\n\
                    Configuration options for each environment listed here:\r\n{}",
                    AvailableEnvironment::values()
                        .into_iter()
                        .map(format_configuration_options)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value("")
                .takes_value(true)
                .value_name("ENVIRONMENT_CONFIGURATION")
                .display_order(15)
            )
            .arg(Arg::with_name("agent")
                .short("a")
                .long("agent")
                .help("specifies the agent to use")
                .long_help(&format!(
                    "Specifies the agent which should be asked. There are limited \
                agents baked into this application. Each agent has its own \
                configuration. See `--agent-configuration` for this.\r\n\r\nCurrently there are \
                {} agents baked into this application:{}\r\n",
                    AvailableAgent::values().len(),
                    AvailableAgent::values()
                        .into_iter()
                        .map(format_available_value)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value(AvailableAgent::Random.nice_name())
                .takes_value(true)
                .hide_possible_values(true)
                .possible_values(
                    &AvailableAgent::values()
                        .into_iter()
                        .map(|a| vec![
                            a.nice_name(), a.short_name(), a.long_name()
                        ].into_iter())
                        .flatten()
                        .collect::<Vec<&str>>()
                )
                .case_insensitive(true)
                .value_name("AGENT")
                .display_order(20)
            )
            .arg(Arg::with_name("agent_configuration")
                .short("b")
                .long("agent-configuration")
                .help("configures the specified agent")
                .long_help(&format!(
                    "Configures the specified agent. The configuration is formatted as \"key=\
                    value;key=value;key=value\" while all additional non formating ';' and '\\' \
                    are escaped with '\\' like \"key=val\\;ue;ke\\;y=va\\\\lue\".\r\n\r\n\
                    Configuration options for each agent listed here:\r\n{}",
                    AvailableAgent::values()
                        .into_iter()
                        .map(format_configuration_options)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value("")
                .takes_value(true)
                .value_name("AGENT_CONFIGURATION")
                .display_order(25)
            )
            .arg(Arg::with_name("visualiser")
                .short("v")
                .long("visualiser")
                .help("specifies the visualiser to utilize")
                .long_help(&format!(
                    "Specifies the visualiser which should be utilized. There are limited \
                visualisers baked into this application. Each visualiser has its own \
                configuration. See `--visualiser-configuration` for this.\r\n\r\nCurrently there \
                are {} visualisers baked into this application:{}\r\n",
                    AvailableVisualiser::values().len(),
                    AvailableVisualiser::values()
                        .into_iter()
                        .map(format_available_value)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value(AvailableVisualiser::None.nice_name())
                .takes_value(true)
                .hide_possible_values(true)
                .possible_values(
                    &AvailableVisualiser::values()
                        .into_iter()
                        .map(|v| vec![
                            v.nice_name(), v.short_name(), v.long_name()
                        ].into_iter())
                        .flatten()
                        .collect::<Vec<&str>>()
                )
                .case_insensitive(true)
                .value_name("VISUALISER")
                .display_order(30)
            )
            .arg(Arg::with_name("visualiser_configuration")
                .short("w")
                .long("visualiser-configuration")
                .help("configures the specified visualiser")
                .long_help(&format!(
                    "Configures the specified visualiser. The configuration is formatted as \"key=\
                    value;key=value;key=value\" while all additional non formating ';' and '\\' \
                    are escaped with '\\' like \"key=val\\;ue;ke\\;y=va\\\\lue\".\r\n\r\n\
                    Configuration options for each visualiser listed here:\r\n{}",
                    AvailableVisualiser::values()
                        .into_iter()
                        .map(format_configuration_options)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value("")
                .takes_value(true)
                .value_name("VISUALISER_CONFIGURATION")
                .display_order(35)
            )
            .arg(Arg::with_name("exit_condition")
                .short("x")
                .long("exit-condition")
                .help("specifies the exit condition to observe")
                .long_help(&format!(
                    "Specifies the exit condition which should be observed. There are limited \
                exit conditions baked into this application. Each exit condition has its own \
                configuration. See `--exit-condition-configuration` for this.\r\n\r\nCurrently \
                there are {} exit conditions baked into this application:{}\r\n",
                    AvailableExitCondition::values().len(),
                    AvailableExitCondition::values()
                        .into_iter()
                        .map(format_available_value)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value(AvailableExitCondition::EpisodesSimulated.nice_name())
                .takes_value(true)
                .hide_possible_values(true)
                .possible_values(
                    &AvailableExitCondition::values()
                        .into_iter()
                        .map(|x| vec![
                            x.nice_name(), x.short_name(), x.long_name()
                        ].into_iter())
                        .flatten()
                        .collect::<Vec<&str>>()
                )
                .case_insensitive(true)
                .value_name("EXIT_CONDITION")
                .display_order(40)
            )
            .arg(Arg::with_name("exit_condition_configuration")
                .short("y")
                .long("exit-condition-configuration")
                .help("configures the specified exit condition")
                .long_help(&format!(
                    "Configures the specified exit condition. The configuration is formatted as \"key=\
                    value;key=value;key=value\" while all additional non formating ';' and '\\' \
                    are escaped with '\\' like \"key=val\\;ue;ke\\;y=va\\\\lue\".\r\n\r\n\
                    Configuration options for each exit condition listed here:\r\n{}",
                    AvailableExitCondition::values()
                        .into_iter()
                        .map(format_configuration_options)
                        .fold(String::new(), |result, line| result + &line)
                ))
                .default_value("")
                .takes_value(true)
                .value_name("EXIT_CONDITION_CONFIGURATION")
                .display_order(45)
            )
            .arg(Arg::with_name("seed")
                .short("s")
                .long("seed")
                .help("sets the seed for initializing the rng")
                .long_help("Sets the seed for initializing the random number generator. This is \
                a string, which gets converted to a list of bytes and then used that way. If no \
                seed is given the seed is chosen randomly.")
                .takes_value(true)
                .value_name("SEED")
                .display_order(50))
            .arg(Arg::with_name("not_reset_environment_on_done")
                .short("r")
                .long("not-reset-environment-on-done")
                .help("does not reset the environment when the environment says it's done")
                .long_help("After every step the environment returns if the current episode is \
                done. With this flag the given environment does not get reset if this happens.")
                .display_order(60))
            .arg(Arg::with_name("reset_agent_on_done")
                .short("q")
                .long("reset-agent-on-done")
                .help("resets the agent when the environment says it's done")
                .long_help("After every step the environment returns if the current episode is \
                done. With this flag the given agent gets reset if this happens.")
                .display_order(70))
            .arg(Arg::with_name("environment_load_path")
                .short("j")
                .long("environment-load-path")
                .help("loads the environment from this file before the start")
                .long_help("Sets the state of the selected environment with the contents of the \
                given file before the loop starts. Be sure to select the corresponding environment \
                to this file. The file format is defined by the file suffix. Currently supported \
                formats are: \"*.json\" (JavaScript Object Notation) and \"*.ron\" (Rusty Object \
                Notation).")
                .takes_value(true)
                .value_name("PATH")
                .display_order(80))
            .arg(Arg::with_name("environment_store_path")
                .short("p")
                .long("environment-store-path")
                .help("stores the environment in this file after exit condition was true")
                .long_help("Saves the state of the selected environment in the given file after \
                the loop stops. The given file will be overwritten. The file format is defined by \
                the file suffix. Currently supported formats are: \"*.json\" (JavaScript Object \
                Notation) and \"*.ron\" (Rusty Object Notation).")
                .takes_value(true)
                .value_name("PATH")
                .display_order(90))
            .arg(Arg::with_name("agent_load_path")
                .short("i")
                .long("agent-load-path")
                .help("loads the agent from this file before the start")
                .long_help("Sets the state of the selected agent with the contents of the \
                given file before the loop starts. Be sure to select the corresponding agent \
                to this file. The file format is defined by the file suffix. Currently supported \
                formats are: \"*.json\" (JavaScript Object Notation) and \"*.ron\" (Rusty Object \
                Notation).")
                .takes_value(true)
                .value_name("PATH")
                .display_order(100))
            .arg(Arg::with_name("agent_store_path")
                .short("o")
                .long("agent-store-path")
                .help("stores the agent in this file after exit condition was true")
                .long_help("Saves the state of the selected agent in the given file after \
                the loop stops. The given file will be overwritten. The file format is defined by \
                the file suffix. Currently supported formats are: \"*.json\" (JavaScript Object \
                Notation) and \"*.ron\" (Rusty Object Notation).")
                .takes_value(true)
                .value_name("PATH")
                .display_order(110)))
        .get_matches();

    if let Some(matched_subcommand_args) = matches.subcommand_matches("command_line") {
        start_with_config(matched_subcommand_args);
    } else if matches.subcommand_matches("interactive").is_some() {
        start_interactively();
    }
}

fn start_with_config(matched_subcommand_args: &ArgMatches) {
    fn split_config(configuration_string: &str) -> HashMap<String, String> {
        let mut output = HashMap::default();
        let mut key = String::new();
        let mut value = String::new();
        let mut currently_parsing_value = false;
        let mut next_escaped = false;
        for c in configuration_string.chars() {
            if !next_escaped && c == '\\' {
                next_escaped = true;
            } else if !next_escaped && !currently_parsing_value && c == '=' {
                currently_parsing_value = true;
            } else if !next_escaped && currently_parsing_value && c == ';' {
                output.insert(key, value);
                key = String::new();
                value = String::new();
                currently_parsing_value = false;
            } else {
                next_escaped = false;
                if currently_parsing_value {
                    value.push(c);
                } else {
                    key.push(c);
                }
            }
        }
        if currently_parsing_value {
            output.insert(key, value);
        }
        output
    }

    let selected_environment = matched_subcommand_args
        .value_of("environment")
        .unwrap()
        .parse::<AvailableEnvironment>()
        .unwrap()
        .select(split_config(
            matched_subcommand_args
                .value_of("environment_configuration")
                .unwrap(),
        ))
        .unwrap();

    let selected_agent = matched_subcommand_args
        .value_of("agent")
        .unwrap()
        .parse::<AvailableAgent>()
        .unwrap()
        .select(split_config(
            matched_subcommand_args
                .value_of("agent_configuration")
                .unwrap(),
        ))
        .unwrap();

    let selected_visualiser = matched_subcommand_args
        .value_of("visualiser")
        .unwrap()
        .parse::<AvailableVisualiser>()
        .unwrap()
        .select(split_config(
            matched_subcommand_args
                .value_of("visualiser_configuration")
                .unwrap(),
        ))
        .unwrap();

    let selected_exit_condition = matched_subcommand_args
        .value_of("exit_condition")
        .unwrap()
        .parse::<AvailableExitCondition>()
        .unwrap()
        .select(split_config(
            matched_subcommand_args
                .value_of("exit_condition_configuration")
                .unwrap(),
        ))
        .unwrap();

    let seed: Option<Seed> = matched_subcommand_args.value_of("seed").map(Seed::from);
    let reset_environment_on_done: bool =
        !matched_subcommand_args.is_present("not_reset_environment_on_done");
    let reset_agent_on_done: bool = matched_subcommand_args.is_present("reset_agent_on_done");
    let environment_load_path: Option<String> = matched_subcommand_args
        .value_of("environment_load_path")
        .map(|string| string.to_string());
    let environment_store_path: Option<String> = matched_subcommand_args
        .value_of("environment_store_path")
        .map(|string| string.to_string());
    let agent_load_path: Option<String> = matched_subcommand_args
        .value_of("agent_load_path")
        .map(|string| string.to_string());
    let agent_store_path: Option<String> = matched_subcommand_args
        .value_of("agent_store_path")
        .map(|string| string.to_string());

    let run_options = RunOptions {
        seed,
        reset_environment_on_done,
        reset_agent_on_done,
        environment_load_path,
        environment_store_path,
        agent_load_path,
        agent_store_path,
    };

    start(
        selected_environment,
        selected_agent,
        selected_visualiser,
        selected_exit_condition,
        run_options,
    );
}

fn start_interactively() {
    println!(
        "{} {}\n\nIn the following steps the necessary configuration values will be collected.",
        APP_NAME,
        crate_version!()
    );

    // ENVIRONMENT
    let selected_environment = select_interactively::<_, AvailableEnvironment, _>(|_| true);
    let selected_environment_supports_visualiser = selected_environment
        .corresponding_available()
        .supports_available();
    let selected_environment_supports_agent = selected_environment
        .corresponding_available()
        .supports_available();
    let selected_environment_supports_exit_condition = selected_environment
        .corresponding_available()
        .supports_available();

    // VISUALISER
    let selected_visualiser = select_interactively::<_, AvailableVisualiser, _>(|available| {
        selected_environment_supports_visualiser.contains(available)
    });
    let selected_visualiser_supports_agent = selected_visualiser
        .corresponding_available()
        .supports_available();
    let selected_visualiser_supports_exit_condition = selected_visualiser
        .corresponding_available()
        .supports_available();

    // AGENT
    let selected_agent = select_interactively::<_, AvailableAgent, _>(|available| {
        selected_environment_supports_agent.contains(available)
            && selected_visualiser_supports_agent.contains(available)
    });
    let selected_agent_supports_exit_condition = selected_agent
        .corresponding_available()
        .supports_available();

    // EXIT CONDITION
    let selected_exit_condition =
        select_interactively::<_, AvailableExitCondition, _>(|available| {
            selected_environment_supports_exit_condition.contains(available)
                && selected_visualiser_supports_exit_condition.contains(available)
                && selected_agent_supports_exit_condition.contains(available)
        });

    // RESET ON DONE
    let reset_environment_on_done = prompt_yes_no(
        "Should the ENVIRONMENT be resetted, when the environment is done after a step?",
        true,
    );

    let reset_agent_on_done = prompt_yes_no(
        "Should the AGENT be resetted, when the environment is done after a step?",
        false,
    );

    // SEED
    let seed =
        prompt_string("Seed for random number generator", None, "randomly chosen").map(Seed::from);

    // LOAD FROM
    let environment_load_path = prompt_string(
        "From which file should the ENVIRONMENT be loaded?",
        None,
        "Do not load",
    );
    let agent_load_path = prompt_string(
        "From which file should the AGENT be loaded?",
        None,
        "Do not load",
    );

    // STORE TO
    let environment_store_path = prompt_string(
        "To which file should the ENVIRONMENT be stored?",
        environment_load_path.clone(),
        "Do not store",
    );
    let agent_store_path = prompt_string(
        "To which file should the AGENT be stored?",
        agent_load_path.clone(),
        "Do not store",
    );

    let run_options = RunOptions {
        seed,
        reset_environment_on_done,
        reset_agent_on_done,
        environment_load_path,
        environment_store_path,
        agent_load_path,
        agent_store_path,
    };

    start(
        selected_environment,
        selected_agent,
        selected_visualiser,
        selected_exit_condition,
        run_options,
    );
}

pub fn prompt_string(
    prompt_text: &str,
    default: Option<String>,
    none_text: &str,
) -> Option<String> {
    println!();
    println!(
        "{} (Default: {})",
        prompt_text,
        match &default {
            Some(s) => s,
            None => none_text,
        }
    );
    print!("> ");
    std::io::stdout().flush().unwrap();

    let mut answer_string = String::new();
    std::io::stdin()
        .read_line(&mut answer_string)
        .expect("Failed to read line");

    if answer_string.trim().is_empty() {
        default
    } else {
        Some(answer_string.trim().to_string())
    }
}

pub fn prompt_yes_no(prompt_text: &str, default: bool) -> bool {
    println!();
    print!(
        "{} ({}) ",
        prompt_text,
        if default { "YES/no" } else { "yes/NO" }
    );
    std::io::stdout().flush().unwrap();

    let mut answer_string = String::new();
    std::io::stdin()
        .read_line(&mut answer_string)
        .expect("Failed to read line");

    if answer_string.trim().is_empty() {
        default
    } else {
        answer_string.trim().to_lowercase().starts_with('y')
    }
}

fn select_interactively<S: Selected<A>, A: Clone + Available<S>, P: Fn(&A) -> bool>(
    predicate: P,
) -> S {
    let (available_elements, unavailable_elements): (Vec<A>, Vec<A>) =
        A::values().into_iter().partition(predicate);
    println!();
    println!("{}", A::category_headline());
    println!("{}", "-".repeat(A::category_headline().len()));
    if available_elements.is_empty() {
        panic!(
            "There are no {} with the previous selections!",
            A::category_headline().to_lowercase()
        );
    }

    for (index, item) in available_elements.iter().enumerate() {
        println!("<{}> {}", index, item.nice_name());
    }

    if !unavailable_elements.is_empty() {
        println!(
            "(Because of your previous choices following elements are not available: {})",
            unavailable_elements
                .into_iter()
                .map(|element| element.nice_name())
                .fold(String::new(), |mut target, name| {
                    if !target.is_empty() {
                        target.push_str(", ");
                    }
                    target.push_str(name);
                    target
                })
        );
    }

    print!("Your choice: ");
    std::io::stdout().flush().unwrap();

    let mut chosen_element_string = String::new();
    std::io::stdin()
        .read_line(&mut chosen_element_string)
        .expect("Failed to read line");

    usize::from_str(chosen_element_string.trim())
        .map_err(|error| format!("{}", error))
        .map(|index| available_elements[index].clone())
        .or_else(|_| {
            chosen_element_string
                .trim()
                .parse::<A>()
                .map_err(|_| format!("Couldn't parse {}", chosen_element_string))
        })
        .and_then(|available| {
            let configuration_options = available.available_configurations();
            let mut chosen_configuration = HashMap::new();
            if !configuration_options.is_empty() {
                println!();
                println!("There are configuration options for your choice. Please answer them.");
                for configuration_option in configuration_options {
                    println!();
                    println!(
                        "{} [{}; default: {}]",
                        configuration_option.name,
                        configuration_option.data_type,
                        configuration_option.default
                    );
                    println!("{}", configuration_option.description);
                    print!("Your answer: ");
                    std::io::stdout().flush().unwrap();

                    let mut answer_string = String::new();
                    std::io::stdin()
                        .read_line(&mut answer_string)
                        .expect("Failed to read line");
                    answer_string = answer_string.trim().to_string();
                    if answer_string.is_empty() {
                        chosen_configuration
                            .insert(configuration_option.name, configuration_option.default);
                    } else {
                        chosen_configuration.insert(configuration_option.name, answer_string);
                    }
                }
            }
            available
                .select(chosen_configuration)
                .map_err(|error| format!("{}", error))
        })
        .unwrap()
}

fn start(
    selected_environment: SelectedEnvironment,
    selected_agent: SelectedAgent,
    selected_visualiser: SelectedVisualiser,
    selected_exit_condition: SelectedExitCondition,
    run_options: RunOptions,
) {
    fn create_environment_gym_mountain_car(goal_velocity: f64) -> MountainCar {
        MountainCar::new(goal_velocity)
    }

    fn create_environment_code_bullet_ai_learns_to_drive(
        sensor_lines_visible: bool,
        track_visible: bool,
    ) -> AiLearnsToDrive {
        let mut a = AiLearnsToDrive::default();
        a.sensor_lines_visible(sensor_lines_visible);
        a.track_visible(track_visible);
        a
    }

    fn create_agent_random(action_spaces: ActionSpace) -> RandomAgent {
        RandomAgent::with(action_spaces)
    }

    fn create_agent_input<
        IP: InputProvider,
        TAMError: Error,
        TAM: ToActionMapper<Vec<input::Input>, TAMError>,
    >(
        input_provider: IP,
        to_action_mapper: TAM,
    ) -> InputAgent<IP, TAMError, TAM> {
        InputAgent::new(input_provider, to_action_mapper)
    }

    fn create_visualiser_piston_in_2d(
        window_title: String,
        window_dimension: (u32, u32),
    ) -> PistonVisualiser {
        PistonVisualiser::run(window_title, window_dimension)
    }

    // INFO: XCF = |environment, agent, episode, step|
    fn create_exit_condition_episodes_simulated_no_visualiser<
        EError: Error,
        EInfo: Debug,
        EData: Serialize + DeserializeOwned,
        E: Environment<EError, EInfo, EData>,
        AError: Error,
        AData: Serialize + DeserializeOwned,
        A: Agent<AError, AData>,
    >(
        count_of_episodes: u128,
    ) -> impl Fn(&E, &A, u128, u128) -> bool {
        move |_environment, _agent, episode, _step| episode >= count_of_episodes
    }

    fn create_exit_condition_episodes_simulated_two_dimensional_visualiser<
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
    >(
        count_of_episodes: u128,
    ) -> impl Fn(&E, &A, &V, u128, u128) -> bool {
        move |_environment, _agent, visualiser, episode, _step| {
            !visualiser.is_open() || episode >= count_of_episodes
        }
    }

    fn create_exit_condition_visualiser_closed_two_dimensional_visualiser<
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
    >() -> impl Fn(&E, &A, &V, u128, u128) -> bool {
        move |_environment, _agent, visualiser, _episode, _step| !visualiser.is_open()
    }

    println!(
        "Starting environment {:?} with agent {:?} within visualiser {:?} and exit condition {:?} \
        using seed {:?}, {}resetting when environment is done and {}resetting when agent is \
        done. Furthermore {} and {}, as well as {} and {}.",
        selected_environment,
        selected_agent,
        selected_visualiser,
        selected_exit_condition,
        run_options.seed.clone().map(|s| s.seed_value),
        if run_options.reset_environment_on_done {
            ""
        } else {
            "not "
        },
        if run_options.reset_agent_on_done {
            ""
        } else {
            "not "
        },
        match &run_options.environment_load_path {
            Some(s) => format!("loading environment from \"{}\"", s),
            None => "not loading environment from file".to_string(),
        },
        match &run_options.environment_store_path {
            Some(s) => format!("storing environment to \"{}\"", s),
            None => "not storing environment to file".to_string(),
        },
        match &run_options.agent_load_path {
            Some(s) => format!("loading agent from \"{}\"", s),
            None => "not loading agent from file".to_string(),
        },
        match &run_options.agent_store_path {
            Some(s) => format!("storing agent to \"{}\"", s),
            None => "not storing agent to file".to_string(),
        },
    );

    match selected_environment {
        SelectedEnvironment::GymMountainCar { goal_velocity } => match selected_agent {
            SelectedAgent::Random => match selected_visualiser {
                SelectedVisualiser::None => match selected_exit_condition {
                    SelectedExitCondition::EpisodesSimulated { count_of_episodes } => {
                        run_with_no_visualiser(
                            create_environment_gym_mountain_car(goal_velocity),
                            create_agent_random(MountainCar::action_space()),
                            create_exit_condition_episodes_simulated_no_visualiser(
                                count_of_episodes,
                            ),
                            run_options,
                        )
                    }
                    SelectedExitCondition::VisualiserClosed => panic!(),
                },
                SelectedVisualiser::PistonIn2d {
                    window_title,
                    window_dimension,
                } => match selected_exit_condition {
                    SelectedExitCondition::EpisodesSimulated { count_of_episodes } => {
                        run_with_two_dimensional_visualiser(
                            create_environment_gym_mountain_car(goal_velocity),
                            create_agent_random(MountainCar::action_space()),
                            create_visualiser_piston_in_2d(window_title, window_dimension),
                            create_exit_condition_episodes_simulated_two_dimensional_visualiser(
                                count_of_episodes,
                            ),
                            run_options,
                        )
                    }
                    SelectedExitCondition::VisualiserClosed => run_with_two_dimensional_visualiser(
                        create_environment_gym_mountain_car(goal_velocity),
                        create_agent_random(MountainCar::action_space()),
                        create_visualiser_piston_in_2d(window_title, window_dimension),
                        create_exit_condition_visualiser_closed_two_dimensional_visualiser(),
                        run_options,
                    ),
                },
            },
            SelectedAgent::Input => match selected_visualiser {
                SelectedVisualiser::None => panic!(),
                SelectedVisualiser::PistonIn2d {
                    window_title,
                    window_dimension,
                } => match selected_exit_condition {
                    SelectedExitCondition::EpisodesSimulated { count_of_episodes } => {
                        let visualiser =
                            create_visualiser_piston_in_2d(window_title, window_dimension);
                        run_with_two_dimensional_visualiser(
                            create_environment_gym_mountain_car(goal_velocity),
                            create_agent_input(
                                visualiser.input_provider(),
                                MountainCarInputToActionMapper::default(),
                            ),
                            visualiser,
                            create_exit_condition_episodes_simulated_two_dimensional_visualiser(
                                count_of_episodes,
                            ),
                            run_options,
                        );
                    }
                    SelectedExitCondition::VisualiserClosed => {
                        let visualiser =
                            create_visualiser_piston_in_2d(window_title, window_dimension);
                        run_with_two_dimensional_visualiser(
                            create_environment_gym_mountain_car(goal_velocity),
                            create_agent_input(
                                visualiser.input_provider(),
                                MountainCarInputToActionMapper::default(),
                            ),
                            visualiser,
                            create_exit_condition_visualiser_closed_two_dimensional_visualiser(),
                            run_options,
                        );
                    }
                },
            },
        },
        SelectedEnvironment::CodeBulletAiLearnsToDrive {
            track_visible,
            sensor_lines_visible,
        } => match selected_agent {
            SelectedAgent::Random => match selected_visualiser {
                SelectedVisualiser::None => match selected_exit_condition {
                    SelectedExitCondition::EpisodesSimulated { count_of_episodes } => {
                        run_with_no_visualiser(
                            create_environment_code_bullet_ai_learns_to_drive(
                                sensor_lines_visible,
                                track_visible,
                            ),
                            create_agent_random(AiLearnsToDrive::action_space()),
                            create_exit_condition_episodes_simulated_no_visualiser(
                                count_of_episodes,
                            ),
                            run_options,
                        )
                    }
                    SelectedExitCondition::VisualiserClosed => panic!(),
                },
                SelectedVisualiser::PistonIn2d {
                    window_title,
                    window_dimension,
                } => match selected_exit_condition {
                    SelectedExitCondition::EpisodesSimulated { count_of_episodes } => {
                        run_with_two_dimensional_visualiser(
                            create_environment_code_bullet_ai_learns_to_drive(
                                sensor_lines_visible,
                                track_visible,
                            ),
                            create_agent_random(AiLearnsToDrive::action_space()),
                            create_visualiser_piston_in_2d(window_title, window_dimension),
                            create_exit_condition_episodes_simulated_two_dimensional_visualiser(
                                count_of_episodes,
                            ),
                            run_options,
                        )
                    }
                    SelectedExitCondition::VisualiserClosed => run_with_two_dimensional_visualiser(
                        create_environment_code_bullet_ai_learns_to_drive(
                            sensor_lines_visible,
                            track_visible,
                        ),
                        create_agent_random(AiLearnsToDrive::action_space()),
                        create_visualiser_piston_in_2d(window_title, window_dimension),
                        create_exit_condition_visualiser_closed_two_dimensional_visualiser(),
                        run_options,
                    ),
                },
            },
            SelectedAgent::Input => {
                match selected_visualiser {
                    SelectedVisualiser::None => panic!(),
                    SelectedVisualiser::PistonIn2d {
                        window_title,
                        window_dimension,
                    } => {
                        match selected_exit_condition {
                            SelectedExitCondition::EpisodesSimulated { count_of_episodes } => {
                                let visualiser =
                                    create_visualiser_piston_in_2d(window_title, window_dimension);
                                run_with_two_dimensional_visualiser(
                            create_environment_code_bullet_ai_learns_to_drive(sensor_lines_visible, track_visible),
                            create_agent_input(
                                visualiser.input_provider(),
                                AiLearnsToDriveInputToActionMapper::default(),
                            ),
                            visualiser,
                            create_exit_condition_episodes_simulated_two_dimensional_visualiser(count_of_episodes),
                            run_options,
                        );
                            }
                            SelectedExitCondition::VisualiserClosed => {
                                let visualiser =
                                    create_visualiser_piston_in_2d(window_title, window_dimension);
                                run_with_two_dimensional_visualiser(
                            create_environment_code_bullet_ai_learns_to_drive(sensor_lines_visible, track_visible),
                            create_agent_input(
                                visualiser.input_provider(),
                                AiLearnsToDriveInputToActionMapper::default(),
                            ),
                            visualiser,
                            create_exit_condition_visualiser_closed_two_dimensional_visualiser(),
                            run_options,
                        );
                            }
                        }
                    }
                }
            }
        },
    }
}
