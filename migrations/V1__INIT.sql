CREATE TABLE jogos (
    id INTEGER PRIMARY KEY NOT NULL,
    nome TEXT NOT NULL
);

CREATE TABLE jogadores (
    id INTEGER PRIMARY KEY NOT NULL,
    nome TEXT NOT NULL,
    email TEXT NOT NULL,
    jogo INTEGER REFERENCES jogos (id) ON DELETE CASCADE
);

CREATE TABLE sorteios (
    id INTEGER PRIMARY KEY NOT NULL,
    seed TEXT NOT NULL,
    jogadores_hash TEXT NOT NULL,
    jogadores_qtd INTEGER NOT NULL,
    jogo INTEGER REFERENCES jogos (id) ON DELETE CASCADE
);

CREATE TABLE envios (
    id INTEGER PRIMARY KEY NOT NULL,
    sorteio INTEGER REFERENCES sorteios (id) NOT NULL,
    destino INTEGER REFERENCES jogadores (id) NOT NULL,
    sorteado INTEGER REFERENCES jogadores (id) NOT NULL,
    sucesso BOOLEAN NOT NULL,
    erro TEXT
);