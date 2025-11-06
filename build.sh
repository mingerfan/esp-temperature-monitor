#!/bin/bash
# 设置工具链路径
export PATH="$PWD/.embuild/espressif/tools/riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/bin:$PATH"

# 执行 cargo 命令
cargo "$@"
