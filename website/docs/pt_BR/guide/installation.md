# Instalação

O ApexSU é exclusivo para dispositivos Android GKI suportados.

## Requisitos

- A versão do kernel deve conter `android`  
  Exemplo: `5.10.209-android12-9-00016-g7c6bbcca33e1`
- `boot.img` correspondente ao firmware/build exato
- ApexSU Manager

## Dispositivos non-GKI

- Não são suportados.
- A instalação é bloqueada.
- Local LKM não é bypass para non-GKI.

## LKM

- Repository LKM: recomendado/padrão para GKI suportado
- Local LKM: opção manual avançada, apenas para GKI suportado

## Aviso de segurança

Modificar boot image pode causar bootloop. Faça backup dos dados importantes antes de instalar.
