build:
	cargo b --release --target aarch64-unknown-none-softfloat -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem
	# download from https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-as ./boot.S -o boot.o
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-ld boot.o target/aarch64-unknown-none-softfloat/release/libminos.rlib -T link.ld -o kernel.elf
dump: 
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objdump ./kernel.elf -D
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objdump ./kernel.elf -s