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
                title: qsTr("About")
            }

            Label {
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                wrapMode: Text.Wrap
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.primaryColor
                text: qsTr("Wasserspiegel shows current water levels from the German PEGELONLINE service. Browse measurement stations, follow water level trends (24 h / 3 d / 7 d) and keep your favourite station on the home screen cover.")
            }

            SectionHeader {
                text: qsTr("Version")
            }

            DetailItem {
                label: qsTr("Version")
                value: "0.2.0"
            }

            SectionHeader {
                text: qsTr("Links")
            }

            BackgroundItem {
                width: parent.width
                onClicked: Qt.openUrlExternally("https://github.com/timgluz/wasserspiegel-sailfish")

                Label {
                    anchors {
                        left: parent.left
                        leftMargin: Theme.horizontalPageMargin
                        verticalCenter: parent.verticalCenter
                    }
                    text: qsTr("Source code (GitHub)")
                    color: Theme.highlightColor
                    font.pixelSize: Theme.fontSizeSmall
                }
            }

            BackgroundItem {
                width: parent.width
                onClicked: Qt.openUrlExternally("https://www.pegelonline.wsv.de")

                Label {
                    anchors {
                        left: parent.left
                        leftMargin: Theme.horizontalPageMargin
                        verticalCenter: parent.verticalCenter
                    }
                    text: qsTr("Data source: PEGELONLINE")
                    color: Theme.highlightColor
                    font.pixelSize: Theme.fontSizeSmall
                }
            }

            BackgroundItem {
                width: parent.width
                onClicked: Qt.openUrlExternally("https://github.com/timgluz/wasserspiegel")

                Label {
                    anchors {
                        left: parent.left
                        leftMargin: Theme.horizontalPageMargin
                        verticalCenter: parent.verticalCenter
                    }
                    text: qsTr("TRMNL dashboard version")
                    color: Theme.highlightColor
                    font.pixelSize: Theme.fontSizeSmall
                }
            }
        }
    }
}
