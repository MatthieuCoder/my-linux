# Simply builds the frontend
build:
	mkdir -p build
	cd buildroot \
		&& make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build browser_linux_defconfig \
		&& make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build
# Cleans the folders
clean:
	rm -r build
# Opens the buildroot menuconfig
menuconfig:
	cd buildroot && make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build menuconfig

.PHONY: build clean