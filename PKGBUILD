pkgname=MediaMagician
pkgver=0.4.0
pkgrel=1
pkgdesc="A very cool file sharing app :)"
arch=('x86_64')
url="https://github.com/Media-Magician/Apps"
license=('MIT')
depends=(
    'cairo'
    'desktop-file-utils'
    'gdk-pixbuf2'
    'glib2'
    'gtk3'
    'hicolor-icon-theme'
    'libsoup'
    'pango'
    'webkit2gtk-4.1'
    'libva'
    'libvdpau'
    'libdrm'
    'libx11'
    'bzip2'
    'xz'
    'x264'
)
options=('!strip' '!emptydirs')
source_x86_64=("${pkgname}-${pkgver}::https://github.com/Media-Magician/Apps/releases/download/v${pkgver}-beta/${pkgname}-x86_64")
sha256sums_x86_64=('03d4cbd5c5a6fa6a40b17c0c30ecb8b34d45789f21f79e0ce0c9d395f6bd6a2c')

package() {
    install -Dm755 "${pkgname}-${pkgver}" "${pkgdir}/usr/bin/${pkgname}"
}
