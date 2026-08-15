import QtQuick 2.0
import Sailfish.Silica 1.0

CoverBackground {
    id: cover

    Column {
        anchors.centerIn: parent
        width: parent.width - Theme.paddingMedium * 2
        spacing: Theme.paddingSmall

        Label {
            text: appController.hasStation ? appController.stationName : qsTr("Wasserspiegel")
            font.pixelSize: Theme.fontSizeMedium
            color: Theme.highlightColor
            anchors.horizontalCenter: parent.horizontalCenter
            truncationMode: TruncationMode.Fade
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
        }

        Label {
            text: appController.hasStation ? appController.water : ""
            font.pixelSize: Theme.fontSizeExtraSmall
            color: Theme.secondaryColor
            anchors.horizontalCenter: parent.horizontalCenter
        }

        Separator {
            width: parent.width
            color: Theme.primaryColor
            horizontalAlignment: Qt.AlignHCenter
        }

        Label {
            text: appController.hasStation && !isNaN(appController.currentLevel)
                  ? Math.round(appController.currentLevel) + " cm"
                  : "–"
            font.pixelSize: Theme.fontSizeHuge
            font.bold: true
            anchors.horizontalCenter: parent.horizontalCenter
        }

        Label {
            text: {
                if (!appController.hasStation || isNaN(appController.change1Day))
                    return ""
                var v = appController.change1Day
                return (v > 0 ? "+" : "") + v.toFixed(1) + " cm (24h)"
            }
            font.pixelSize: Theme.fontSizeExtraSmall
            color: {
                if (!appController.hasStation || isNaN(appController.change1Day))
                    return Theme.secondaryColor
                return appController.change1Day > 0 ? "#4caf50" : "#f44336"
            }
            anchors.horizontalCenter: parent.horizontalCenter
        }

        Label {
            text: appController.fromCache ? qsTr("offline") : ""
            font.pixelSize: Theme.fontSizeTiny
            color: Theme.secondaryColor
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }

    CoverActionList {
        id: coverAction

        CoverAction {
            iconSource: "image://theme/icon-s-sync"
            onTriggered: appController.refresh()
        }
    }
}
