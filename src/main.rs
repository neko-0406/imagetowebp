use clap::Parser;
use imagetowebp::cli::{CliArgs, prompt_interactive_config};
use imagetowebp::config::ConvertConfig;

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let config: ConvertConfig = if !args.input.is_empty() {
        args.to_config()
    } else {
        prompt_interactive_config()?
    };

    config.validate().map_err(|e| anyhow::anyhow!(e))?;

    println!("Config loaded successfully: {:?}", config);
    Ok(())
}
