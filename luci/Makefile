# OpenWrt Makefile for LuCI NetPath application
include $(TOPDIR)/rules.mk

PKG_NAME:=luci-app-netpath
PKG_VERSION:=1.0.0
PKG_RELEASE:=1

PKG_LICENSE:=GPL-2.0
PKG_MAINTAINER:=Hikaru <i@rua.moe>

LUCI_TITLE:=LuCI support for NetPath
LUCI_DEPENDS:=+netpath +luci-base +luci-mod-admin-full +luci-lib-jsonc
LUCI_PKGARCH:=all

PKG_BUILD_DIR:=$(BUILD_DIR)/$(PKG_NAME)

include $(TOPDIR)/feeds/luci/luci.mk

define Package/luci-app-netpath/description
	LuCI web interface for NetPath multi-WAN manager.
	Provides configuration and monitoring capabilities.
	.
	Compatible with OpenWrt 22.03+ and LuCI 23.05+.
endef

# vim: set noexpandtab:
