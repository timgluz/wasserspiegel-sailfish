import QtQuick 2.0
import Sailfish.Silica 1.0

Page {
    id: page

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: content.height + Theme.paddingLarge

        Column {
            id: content
            width: parent.width
            spacing: Theme.paddingMedium

            PageHeader {
                title: qsTr("Logs")
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingMedium

                Button {
                    text: qsTr("Refresh")
                    onClicked: refreshLog()
                }
                Button {
                    text: qsTr("Copy")
                    onClicked: {
                        Clipboard.text = logLabel.text
                        banner.notify(qsTr("Logs copied to clipboard"))
                    }
                }
            }

            Label {
                id: logLabel
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                font.pixelSize: Theme.fontSizeExtraSmall
                color: Theme.secondaryColor
                wrapMode: Text.Wrap
                textFormat: Text.PlainText
                text: ""
            }
        }
    }

    Timer {
        interval: 2000
        running: status === PageStatus.Active
        repeat: true
        onTriggered: refreshLog()
    }

    Banner {
        id: banner
    }

    function refreshLog() {
        logLabel.text = appController.logText()
    }

    Component.onCompleted: refreshLog()
}
