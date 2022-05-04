# download tools from https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads
# install qemu-arch-extra on arch based distros

build:
	cargo b --release
	./tools/aarch64-none-elf-objcopy target/aarch64-raspi3/release/minos kernel.elf
	./tools/aarch64-none-elf-objcopy kernel.elf -O binary kernel8.img
qemu: build
	qemu-system-aarch64 -M raspi3b --serial stdio --kernel kernel8.img
deploy: build
	sudo mount /dev/sdb1 /home/sampo/sd/
	sudo cp kernel8.img /home/sampo/sd/
	sudo umount /dev/sdb1
dump: build
	./tools/aarch64-none-elf-objdump target/aarch64-raspi3/release/minos -D > dump.txt
clean:
	cargo clean
	rm kernel.elf
	rm kernel8.img
