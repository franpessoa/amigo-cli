use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Clone, Subcommand, Debug)]
pub enum Commands {
    Jogo {
        #[command(subcommand)]
        action: JogoAction,
    },
    Jogadores {
        #[command(subcommand)]
        action: JogadoresAction,
    },
    Sorteio {
        #[command(subcommand)]
        action: SorteioAction,
    },
    Envio {
        #[command(subcommand)]
        action: EnvioAction,
    },
}

#[derive(Clone, Subcommand, Debug)]
pub enum JogoAction {
    New {
        name: String,
    },
    Rm {
        id: u64,
    },
    Ls,
    From {
        #[arg(short, long, default_value = "csv")]
        format: JogoFromFormat,

        #[arg(short, long)]
        path: PathBuf,

        #[arg(short, long)]
        nome: String,
    },
}

#[derive(Clone, Subcommand, Debug)]
pub enum JogadoresAction {
    Add {
        #[arg(short, long)]
        jogo: u64,

        nome: String,
        email: String,
    },
    Rm {
        jogador: u64,
    },
    Inspect {
        jogador: u64,
    },
    Ls {
        #[arg(short, long, default_value=None)]
        jogo: Option<u64>,
    },
    Set {
        id: u64,

        #[command(subcommand)]
        param: JogadoresSetParams,
    },
}

#[derive(Clone, Subcommand, Debug)]
pub enum JogadoresSetParams {
    Nome { val: String },
    Email { val: String },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum JogoFromFormat {
    Csv,
}

#[derive(Clone, Subcommand, Debug)]
pub enum SorteioAction {
    New {
        jogo: u64,
    },
    Run {
        sorteio: u64,
    },
    Ls {
        #[arg(short, long, default_value=None)]
        jogo: Option<u64>,
    },
    Inspect {
        sorteio: u64,
    },
}

#[derive(Clone, Subcommand, Debug)]
pub enum EnvioAction {
    Ls {
        #[arg(short, long, default_value=None)]
        sorteio: Option<u64>,
    },
    Inspect {
        envio: u64,
    },
    Redo {
        envio: u64,
    },
}

#[derive(Debug, Parser)]
pub struct Arguments {
    #[command(subcommand)]
    pub cmd: Commands,

    #[arg(short, long, default_value = "false")]
    pub debug: bool,
}
