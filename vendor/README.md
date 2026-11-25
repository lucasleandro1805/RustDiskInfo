# Código-fonte empacotado

O arquivo smartmontools-7.5.tar.gz é a distribuição oficial de código-fonte do smartmontools 7.5:

https://github.com/smartmontools/smartmontools/releases/download/RELEASE_7_5/smartmontools-7.5.tar.gz

SHA-256: 690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e

scripts/build.sh verifica o hash, compila smartctl a partir deste arquivo e o coloca ao lado do RustDiskInfo. O script também instala a base de modelos drivedb.h.
