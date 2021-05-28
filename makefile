qemu:
	cargo b --release
	aarch64-none-elf-objcopy target/aarch64-raspi3/release/minos kernel.elf
	aarch64-none-elf-objcopy kernel.elf -O binary kernel8.img
	qemu-system-aarch64 -M raspi3 --serial stdio --kernel kernel8.img
	# download from https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads
deploy:
	cargo b --release
	aarch64-none-elf-objcopy target/aarch64-raspi3/release/minos kernel.elf
	aarch64-none-elf-objcopy kernel.elf -O binary kernel8.img
	sudo mount /dev/sdb1 /home/sampo/sd/
	sudo cp kernel8.img /home/sampo/sd/
	sudo umount /dev/sdb1
dump:
	aarch64-none-elf-objdump target/aarch64-raspi3/release/minos -D > dump.txt
clean:
	cargo clean
	rm kernel.elf
	rm kernel8.img
