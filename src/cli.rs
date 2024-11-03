use clap::{Parser, Subcommand};

#[derive(Clone, Subcommand)]
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
}

#[derive(Clone, Subcommand)]
pub enum JogoAction {
    New { name: String },
    Rm { id: u64 },
    Ls,
}

#[derive(Clone, Subcommand)]
pub enum JogadoresAction {
    Add {
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
        jogo: u64,
    },
    Set {
        id: u64,

        #[command(subcommand)]
        param: JogadoresSetParams,
    },
}

#[derive(Clone, Subcommand)]
pub enum JogadoresSetParams {
    Nome { val: String },
    Email { val: String },
}

#[derive(Clone, Subcommand)]
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

#[derive(Parser)]
pub struct Arguments {
    #[command(subcommand)]
    pub cmd: Commands,
}
