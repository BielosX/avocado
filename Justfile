toolchain:
    rustup target add thumbv7em-none-eabihf

build: toolchain
    cargo build --release
    cp target/thumbv7em-none-eabihf/release/avocado target/thumbv7em-none-eabihf/release/avocado.elf

sections:
    objdump -h target/thumbv7em-none-eabihf/release/avocado

vector-table:
    objdump -s -j .vector_table target/thumbv7em-none-eabihf/release/avocado

flash: clean build
    $STM32_PROGRAMMER_CLI -c port=SWD -d target/thumbv7em-none-eabihf/release/avocado.elf -rst

clean:
    cargo clean