use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Error, Result};
use clap::ValueEnum;
use indexmap::IndexSet;
use path_absolutize::Absolutize;

use crate::{compose, config::Config};

/// Converts the Compose file to platform's canonical format
#[derive(clap::Args, Debug)]
#[command(alias = "config", next_display_order = None)]
pub(crate) struct Args {
    /// Format the output
    #[arg(long, value_enum, default_value_t = Format::Yaml)]
    format: Format,

    /// Only validate the configuration, don't print anything
    #[arg(short, long)]
    quiet: bool,

    /// Don't interpolate environment variables
    #[arg(long)]
    no_interpolate: bool,

    /// Print the service names, one per line
    #[arg(long)]
    services: bool,

    /// Print the volume names, one per line
    #[arg(long)]
    volumes: bool,

    /// Print the profile names, one per line
    #[arg(long)]
    profiles: bool,

    /// Print the image names, one per line
    #[arg(long)]
    images: bool,

    /// Save to file (default to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
enum Format {
    Yaml,
    Json,
}

pub(crate) fn run(args: Args, config: Config) -> Result<()> {
    let file = compose::parse(&config, args.no_interpolate)?;

    if !args.quiet {
        if args.services {
            for service in file.services.into_keys() {
                println!("{service}");
            }
        } else if args.volumes {
            for volume in file.volumes.into_keys() {
                println!("{volume}");
            }
        } else if args.profiles {
            let mut all_profiles = IndexSet::new();

            for service in file.services.into_values() {
                all_profiles.extend(service.profiles);
            }

            for profile in all_profiles {
                println!("{profile}");
            }
        } else if args.images {
            for service in file.services.into_values() {
                if let Some(image) = service.image {
                    println!("{image}");
                }
            }
        } else {
            match args.format {
                Format::Yaml => {
                    let contents = serde_yaml::to_string(&file)?;

                    if let Some(path) = args.output {
                        fs::write(&path, contents).with_context(|| match path.absolutize() {
                            Ok(path) => anyhow!(
                                "{} not found",
                                path.parent().unwrap_or_else(|| Path::new("/")).display()
                            ),
                            Err(err) => Error::from(err),
                        })?;
                    } else {
                        print!("{contents}");
                    }
                }
                Format::Json => {
                    let mut contents = serde_json::to_string_pretty(&file)?;
                    contents.push('\n');

                    if let Some(path) = args.output {
                        fs::write(&path, contents).with_context(|| match path.absolutize() {
                            Ok(path) => anyhow!(
                                "{} not found",
                                path.parent().unwrap_or_else(|| Path::new("/")).display()
                            ),
                            Err(err) => Error::from(err),
                        })?;
                    } else {
                        print!("{contents}");
                    }
                }
            }
        }
    }

    Ok(())
}
