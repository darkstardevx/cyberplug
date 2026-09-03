import QtQuick
import qs.Commons
import qs.Ui

BarWidget {
  id: root
  moduleName: "custom.cyberplug"

  function launch() {
    if (!root.bar) return
    root.bar.run("cyberplug-toggle")
  }

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  WidgetButton {
    id: button
    bar: root.bar
    text: "CP"
    horizontalMargin: 6
    verticalPadding: 6
    fixedWidth: root.vertical ? root.barSize : Style.space(20)
    fixedHeight: root.barSize
    onPressed: root.launch()
  }
}
