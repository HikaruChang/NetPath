# OpenWrt Makefile for NetPath
include $(TOPDIR)/rules.mk

PKG_NAME:=netpath
PKG_VERSION:=1.0.0
PKG_RELEASE:=1

PKG_SOURCE_PROTO:=git
PKG_SOURCE_URL:=https://github.com/hikaruchang/netpath.git
PKG_SOURCE_VERSION:=HEAD
PKG_MIRROR_HASH:=skip

PKG_MAINTAINER:=Hikaru <i@rua.moe>
PKG_LICENSE:=GPL-2.0
PKG_LICENSE_FILES:=LICENSE

PKG_BUILD_DEPENDS:=rust/host
PKG_BUILD_PARALLEL:=1

include $(INCLUDE_DIR)/package.mk
include $(TOPDIR)/feeds/packages/lang/rust/rust-package.mk

define Package/netpath
  SECTION:=net
  CATEGORY:=Network
  TITLE:=High-performance multi-WAN manager
  URL:=https://github.com/hikaruchang/netpath
  DEPENDS:=+nftables +kmod-nft-core +kmod-nft-nat +kmod-nft-fib \
    +ip-full +ipset +jsonfilter +curl +bind-dig \
    +coreutils +coreutils-timeout +netcat
  PKGARCH:=all
endef

define Package/netpath/description
  NetPath is a modern multi-WAN load balancing and failover manager
  for OpenWrt. It's a complete rewrite of mwan3 using Rust and nftables
  for superior performance and efficiency.
  .
  Compatible with OpenWrt 22.03, 23.05, and later versions.
endef

define Package/netpath/conffiles
/etc/config/netpath
/etc/netpath/config.toml
endef

# Rust build configuration
CARGO_VARS += \
	CARGO_HOME=$(CARGO_HOME)

define Build/Compile
	cd $(PKG_BUILD_DIR)/core && \
	$(CARGO_VARS) \
	cargo build \
		--release \
		--target=$(RUSTC_TARGET_ARCH) \
		--manifest-path=Cargo.toml
endef

define Package/netpath/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/core/target/$(RUSTC_TARGET_ARCH)/release/netpath $(1)/usr/bin/
	
	$(INSTALL_DIR) $(1)/etc/netpath
	$(INSTALL_DATA) $(PKG_BUILD_DIR)/core/examples/config.toml $(1)/etc/netpath/
	
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_DATA) $(PKG_BUILD_DIR)/core/files/netpath.uci $(1)/etc/config/netpath
	
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/core/files/netpath.init $(1)/etc/init.d/netpath
	
	$(INSTALL_DIR) $(1)/etc/hotplug.d/iface
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/core/files/netpath.hotplug $(1)/etc/hotplug.d/iface/15-netpath
endef

define Package/netpath/postinst
#!/bin/sh
[ -n "$${IPKG_INSTROOT}" ] || {
	/etc/init.d/netpath enable
	# Check nftables support
	if ! which nft >/dev/null 2>&1; then
		echo "Warning: nftables not found. Please install nftables package."
	fi
	echo "NetPath installed. Configure via /etc/config/netpath or LuCI."
	echo "Note: Compatible with OpenWrt 22.03, 23.05, and later."
}
exit 0
endef

define Package/netpath/prerm
#!/bin/sh
[ -n "$${IPKG_INSTROOT}" ] || {
	/etc/init.d/netpath stop
	/etc/init.d/netpath disable
}
exit 0
endef

$(eval $(call BuildPackage,netpath))
