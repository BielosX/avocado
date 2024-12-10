arch := "thumbv7em-none-eabihf"

toolchain:
    rustup target add {{arch}}

build: toolchain
    cargo build --release
    cp target/{{arch}}/release/avocado target/{{arch}}/release/avocado.elf

sections:
    objdump -h target/{{arch}}/release/avocado

vector-table:
    objdump -s -j .vector_table target/{{arch}}/release/avocado

flash: clean build
    $STM32_PROGRAMMER_CLI -c port=SWD -d target/{{arch}}/release/avocado.elf -rst

fmt:
    cargo fmt

clean:
    cargo clean

usart-watch:
    sudo minicom -b 115200 -o -D /dev/ttyACM0
