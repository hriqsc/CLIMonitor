# CLIMonitor

Uma versão de terminal do WebMonitor do Protheus, projetada para fornecer uma experiência de monitoramento e gerenciamento de conexões do protheus atravez do terminal.

## Funcionalidades

* Monitoramento de conexões de rede em tempo real, similar ao WebMonitor
* Gerenciamento de conexões do serviço protheus
* Interface de usuário no terminal, leve e portatil

## Diferenças em relação ao WebMonitor

* Interface de usuário no terminal, em vez de uma interface web
* Possibilidade de executar comandos e gerenciar conexões de forma mais rápida e eficiente

## Sobre o CLIMonitor

O CLIMonitor é uma ferramenta feita em rust para fins de melhor controle de conexões em serviços

## Dependências

* `reqwest` para fazer requisições HTTP
* `tokio` para gerenciar a execução assíncrona
* `ratatui` para criar a interface de usuário em linha de comando
* `serde` para serializar e deserializar dados

## Licença

Este projeto é licenciado sob a licença MIT. Consulte o arquivo `LICENSE` para mais informações.