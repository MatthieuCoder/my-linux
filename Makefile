build:
	mkdir -p build
	cd buildroot && make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build browser_linux_defconfig

	cd buildroot && make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build  
clean:
	rm -r build

print_var:
	echo $(PWD)
menuconfig:
	cd buildroot && make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build menuconfig
upd:
	cd buildroot && make BR2_EXTERNAL=$(PWD)/matthieunux O=$(PWD)/build browser_linux_defconfig
.PHONY: build clean