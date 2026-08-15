import QtQuick 2.0
import Sailfish.Silica 1.0

Page {
    id: page

    readonly property bool hasData: appController.seriesPoints.length > 1

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: content.height

        PullDownMenu {
            busy: appController.loading
            MenuItem {
                text: qsTr("Settings")
                onClicked: pageStack.push(Qt.resolvedUrl("SettingsPage.qml"))
            }
            MenuItem {
                text: qsTr("Change station")
                onClicked: pageStack.push(Qt.resolvedUrl("StationPickerPage.qml"))
            }
            MenuItem {
                text: qsTr("Refresh")
                onClicked: appController.refresh()
            }
            MenuItem {
                text: qsTr("Logs")
                onClicked: pageStack.push(Qt.resolvedUrl("LogsPage.qml"))
            }
            MenuItem {
                text: qsTr("About")
                onClicked: pageStack.push(Qt.resolvedUrl("AboutPage.qml"))
            }
        }

        Column {
            id: content
            width: parent.width
            spacing: Theme.paddingMedium

            PageHeader {
                title: appController.stationName !== ""
                       ? (appController.stationName + " / " + appController.water)
                       : qsTr("Wasserspiegel")
            }

            // ---- config banner (tap to open Settings) ----

            BackgroundItem {
                visible: appController.needsConfig
                width: parent.width
                height: Theme.itemSizeSmall

                onClicked: pageStack.push(Qt.resolvedUrl("SettingsPage.qml"))

                Label {
                    anchors {
                        left: parent.left
                        leftMargin: Theme.horizontalPageMargin
                        right: parent.right
                        rightMargin: Theme.horizontalPageMargin
                        verticalCenter: parent.verticalCenter
                    }
                    text: qsTr("API not configured - tap to set up")
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.highlightColor
                    wrapMode: Text.Wrap
                }
            }

            // ---- current level ----

            Item {
                width: parent.width
                height: Theme.itemSizeHuge

                Label {
                    anchors.centerIn: parent
                    text: isNaN(appController.currentLevel)
                          ? "–"
                          : Math.round(appController.currentLevel) + " " + appController.unit
                    font.pixelSize: Theme.fontSizeHuge * 1.6
                    font.bold: true
                    color: Theme.highlightColor
                }
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingLarge

                Label {
                    text: formatDelta(appController.change1Day)
                    color: deltaColor(appController.change1Day)
                    font.pixelSize: Theme.fontSizeMedium
                }
                Label {
                    text: formatDelta(appController.change3Day)
                    color: deltaColor(appController.change3Day)
                    font.pixelSize: Theme.fontSizeMedium
                }
                Label {
                    text: formatDelta(appController.change7Day)
                    color: deltaColor(appController.change7Day)
                    font.pixelSize: Theme.fontSizeMedium
                }
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingLarge + Theme.paddingMedium
                Label {
                    text: qsTr("24 h")
                    color: Theme.secondaryHighlightColor
                    font.pixelSize: Theme.fontSizeExtraSmall
                }
                Label {
                    text: qsTr("3 d")
                    color: Theme.secondaryHighlightColor
                    font.pixelSize: Theme.fontSizeExtraSmall
                }
                Label {
                    text: qsTr("7 d")
                    color: Theme.secondaryHighlightColor
                    font.pixelSize: Theme.fontSizeExtraSmall
                }
            }

            // ---- demo data hint ----

            Label {
                visible: !appController.hasStation
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                font.pixelSize: Theme.fontSizeExtraSmall
                color: Theme.secondaryColor
                text: qsTr("Sample data - pick a station for live readings")
            }

            // ---- offline / error notices ----

            Label {
                visible: appController.fromCache && !appController.loading
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                font.pixelSize: Theme.fontSizeExtraSmall
                color: Theme.secondaryColor
                text: qsTr("Offline - showing data from %1")
                        .arg(Format.formatDate(new Date(appController.lastUpdatedMs), Formatter.Timepoint))
            }

            Label {
                visible: appController.error !== "" && !appController.loading
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                font.pixelSize: Theme.fontSizeExtraSmall
                color: Theme.errorColor
                text: appController.error
            }

            // ---- period selector ----

            Row {
                id: periodRow
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.paddingSmall

                property var periods: [
                    { label: qsTr("24 h"), hours: 24 },
                    { label: qsTr("3 d"), hours: 72 },
                    { label: qsTr("10 d"), hours: 240 }
                ]

                Repeater {
                    model: periodRow.periods
                    delegate: BackgroundItem {
                        height: Theme.itemSizeExtraSmall
                        width: Theme.itemSizeExtraSmall * 2

                        Rectangle {
                            anchors.fill: parent
                            radius: Theme.paddingSmall
                            color: appController.graphPeriodHours === modelData.hours
                                   ? Theme.highlightBackgroundColor
                                   : Theme.secondaryHighlightColor
                            opacity: appController.graphPeriodHours === modelData.hours ? 0.8 : 0.2
                        }

                        Label {
                            anchors.centerIn: parent
                            text: modelData.label
                            font.pixelSize: Theme.fontSizeSmall
                            color: appController.graphPeriodHours === modelData.hours
                                   ? Theme.highlightColor
                                   : Theme.primaryColor
                        }

                        onClicked: appController.graphPeriodHours = modelData.hours
                    }
                }
            }

            // ---- trend graph ----

            TrendGraph {
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                height: Theme.itemSizeLarge * 3
                points: appController.seriesPoints
                minValue: appController.seriesMin
                maxValue: appController.seriesMax
                startMs: appController.seriesStartMs
                endMs: appController.seriesEndMs
                unit: appController.unit
            }

            // ---- details ----

            DetailItem {
                label: qsTr("River")
                value: appController.water
            }
            DetailItem {
                label: qsTr("Station")
                value: appController.stationName
            }
            DetailItem {
                label: qsTr("from source")
                value: appController.km > 0 ? appController.km.toFixed(1) : "-"
            }
            DetailItem {
                label: qsTr("Last measurement")
                value: appController.lastUpdatedMs > 0
                       ? Format.formatDate(new Date(appController.lastUpdatedMs), Formatter.Timepoint)
                       : "-"
            }
        }
    }

    BusyIndicator {
        anchors.centerIn: parent
        running: appController.loading && !page.hasData
        size: BusyIndicatorSize.Large
    }

    function formatDelta(v) {
        if (isNaN(v)) return "-"
        var s = v > 0 ? "+" : ""
        return s + v.toFixed(1) + " " + appController.unit
    }

    function deltaColor(v) {
        if (isNaN(v)) return Theme.secondaryColor
        return v > 0 ? "#4caf50" : (v < 0 ? "#f44336" : Theme.secondaryColor)
    }
}
