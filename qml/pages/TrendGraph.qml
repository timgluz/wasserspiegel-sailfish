import QtQuick 2.0
import Sailfish.Silica 1.0

Item {
    id: graph

    property var points: []
    property real minValue: 0
    property real maxValue: 1
    property double startMs: 0
    property double endMs: 0
    property string unit: "cm"

    readonly property real leftMargin: Theme.fontSizeSmall * 3
    readonly property real bottomMargin: Theme.fontSizeSmall * 1.6
    readonly property real topPadding: Theme.paddingSmall

    Canvas {
        id: canvas
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
        }
        height: parent.height - graph.bottomMargin

        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()

            var w = width - graph.leftMargin
            var h = height - 2 * graph.topPadding
            if (w <= 0 || h <= 0) return

            var range = graph.maxValue - graph.minValue
            if (range <= 0) range = 1

            var pts = graph.points
            if (pts.length < 2) return

            function px(i) { return graph.leftMargin + pts[i].x * w }
            function py(i) {
                return graph.topPadding + (h - (pts[i].y - graph.minValue) / range * h)
            }

            // subtle fill under the line
            ctx.beginPath()
            ctx.moveTo(px(0), height)
            for (var i = 0; i < pts.length; i++) ctx.lineTo(px(i), py(i))
            ctx.lineTo(px(pts.length - 1), height)
            ctx.closePath()
            ctx.fillStyle = Theme.rgba(Theme.highlightColor, 0.15)
            ctx.fill()

            // the line itself
            ctx.beginPath()
            for (var j = 0; j < pts.length; j++) {
                if (j === 0) ctx.moveTo(px(j), py(j))
                else ctx.lineTo(px(j), py(j))
            }
            ctx.lineWidth = 2
            ctx.strokeStyle = Theme.highlightColor
            ctx.stroke()

            // latest value dot
            ctx.beginPath()
            ctx.arc(px(pts.length - 1), py(pts.length - 1), 4, 0, Math.PI * 2)
            ctx.fillStyle = Theme.highlightColor
            ctx.fill()
        }
    }

    // min/max labels on the left
    Label {
        anchors { top: parent.top; topMargin: graph.topPadding; left: parent.left }
        font.pixelSize: Theme.fontSizeExtraSmall
        color: Theme.secondaryColor
        text: graph.maxValue.toFixed(0)
    }
    Label {
        anchors { bottom: canvas.bottom; left: parent.left }
        font.pixelSize: Theme.fontSizeExtraSmall
        color: Theme.secondaryColor
        text: graph.minValue.toFixed(0)
    }

    // time axis labels
    Label {
        anchors { left: parent.left; leftMargin: graph.leftMargin; bottom: parent.bottom }
        font.pixelSize: Theme.fontSizeExtraSmall
        color: Theme.secondaryColor
        text: graph.startMs > 0
              ? Format.formatDate(new Date(graph.startMs), Formatter.Timepoint)
              : ""
    }
    Label {
        anchors { right: parent.right; bottom: parent.bottom }
        font.pixelSize: Theme.fontSizeExtraSmall
        color: Theme.secondaryColor
        text: graph.endMs > 0
              ? Format.formatDate(new Date(graph.endMs), Formatter.Timepoint)
              : ""
    }

    onPointsChanged: canvas.requestPaint()
    onMinValueChanged: canvas.requestPaint()
    onMaxValueChanged: canvas.requestPaint()
}
