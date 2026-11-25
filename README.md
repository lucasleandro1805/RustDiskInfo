<div align="center">
  <img src="assets/rustdiskinfo-icon.png" alt="Ícone do RustDiskInfo: disco com anel verde de saúde" width="160">

  # RustDiskInfo

  **Saúde dos seus discos, de relance.**

  Monitor S.M.A.R.T. para Linux, feito em Rust com egui e inspirado no CrystalDiskInfo e no QDiskInfo.

  ![Linux](https://img.shields.io/badge/Linux-desktop-222?logo=linux&logoColor=white)
  ![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)
  ![egui](https://img.shields.io/badge/UI-egui-65BE98)
  ![License](https://img.shields.io/badge/license-GPL--3.0-blue)
</div>

---

## Visão geral

O RustDiskInfo apresenta informações de saúde e atributos S.M.A.R.T. das unidades detectadas pelo smartctl, em uma interface desktop escura e direta.

Ao abrir, o app mostra uma tela inicial com o resumo das unidades detectadas e um botão para entrar no monitor. O atalho criado pelo instalador aparece no menu de aplicativos com o ícone do RustDiskInfo.

### O que mostra

- Unidades detectadas e modelo
- Estado geral de saúde S.M.A.R.T.
- Temperatura e horas de funcionamento
- Capacidade, número de série e protocolo
- Tabela de atributos S.M.A.R.T. ATA
- Atualização manual dos dados

## Requisitos

- Linux com ambiente gráfico compatível com X11
- Rust e Cargo para compilar
- smartctl disponível na máquina que prepara a instalação
- Permissões do sistema para ler os dispositivos S.M.A.R.T. (alguns discos exigem acesso elevado)

O instalador copia o executável smartctl para ~/.local/bin junto com o RustDiskInfo. Assim, após a instalação, o app procura primeiro essa cópia ao lado do próprio executável.

## Compilar

    ./scripts/build.sh

O binário de release é criado em build/release/rustdiskinfo.

## Instalar

    ./scripts/install.sh

O script instala o app e o smartctl em ~/.local/bin e cria um atalho no menu de aplicativos. Não configura inicialização automática.

Para instalar um binário já compilado, passe o caminho como argumento:

    ./scripts/install.sh /caminho/para/rustdiskinfo

## Estado do projeto

Esta é a primeira versão. A leitura S.M.A.R.T. depende do formato JSON fornecido pelo smartctl; os atributos detalhados disponíveis variam conforme a unidade e a controladora.

## Créditos e licenças

A interface e o fluxo de informação foram inspirados pelo [QDiskInfo](https://github.com/edisionnano/QDiskInfo), projeto GPL-3.0.

O instalador pode incluir uma cópia do smartctl, parte do [smartmontools](https://github.com/smartmontools/smartmontools), licenciado sob GPL-2.0-or-later. Os créditos e avisos estão em [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Este projeto está licenciado sob [GNU GPL-3.0-or-later](LICENSE).
