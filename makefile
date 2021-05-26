qemu:
	cargo b --release
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objcopy target/aarch64-raspi3/release/minos kernel.elf
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objcopy kernel.elf -O binary kernel8.img
	qemu-system-aarch64 -M raspi3 --serial stdio --kernel kernel8.img
	# download from https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads
deploy:
	cargo b --release
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objcopy target/aarch64-raspi3/release/minos kernel.elf
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objcopy kernel.elf -O binary kernel8.img
	sudo cp kernel8.img /home/sampo/sd/
dump:
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objdump target/aarch64-raspi3/release/minos -D
	/home/sampo/Downloads/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf/bin/aarch64-none-elf-objdump target/aarch64-raspi3/release/minos -s
clean:
	cargo clean
	rm kernel.elf
	rm kernel8.img