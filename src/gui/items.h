// GUI scene item classes that the canvas places into QGraphicsScene.
//
// Currently extracted: WireItem (orthogonal wire path + bus annotation) and
// JunctionDotItem (T-tap dot). Other item classes (PortPinItem, BundlePinItem,
// InstanceItem, TopPortItem) still live in app.cpp pending further split —
// they touch WireTool / CanvasLayer / dialogs which haven't moved yet.

#pragma once

#include "canvas_constants.h"

#include <QColor>
#include <QGraphicsEllipseItem>
#include <QGraphicsPathItem>
#include <QGraphicsSceneContextMenuEvent>
#include <QInputDialog>
#include <QLineEdit>
#include <QMenu>
#include <QPainter>
#include <QPainterPath>
#include <QPainterPathStroker>
#include <QPen>
#include <QPointF>
#include <QString>
#include <QStyleOptionGraphicsItem>
#include <QVector>
#include <cmath>

#include "hdl-compose/src/gui/bridge.cxxqt.h"

namespace hdlc {

// Filled circle marking a same-net junction (T-tap of two or more wires).
class JunctionDotItem : public QGraphicsEllipseItem {
  public:
    JunctionDotItem(const QPointF &center, const QColor &color)
        : QGraphicsEllipseItem(center.x() - kJunctionDotRadius, center.y() - kJunctionDotRadius,
                               2 * kJunctionDotRadius, 2 * kJunctionDotRadius) {
        setBrush(color);
        QPen pen(color);
        pen.setCosmetic(true);
        setPen(pen);
        setZValue(2);
    }
};

// Orthogonal wire path between two pin keys ("inst.port" or "top:port"),
// rendered with a bus-width slash + bit count for multi-bit nets. The
// CanvasLayer planner sets the path via setWaypoints; WireItem stores the
// raw waypoints so junction-dot scanning and selection can introspect.
class WireItem : public QGraphicsPathItem {
  public:
    WireItem(const QString &source_key, const QString &target_key)
        : m_source_key(source_key), m_target_key(target_key) {
        setFlag(QGraphicsItem::ItemIsSelectable, true);
        setAcceptedMouseButtons(Qt::LeftButton | Qt::RightButton);
        QPen pen(colorForNet(source_key), 1.5);
        pen.setCosmetic(true);
        setPen(pen);
    }

    // Hash the net identity to a deterministic hue. Muted palette so wires
    // read as background detail rather than competing with module bodies.
    static QColor colorForNet(const QString &key) {
        int hue = static_cast<int>(qHash(key) % 360);
        return QColor::fromHsv(hue, 110, 200);
    }

    const QString &sourceKey() const { return m_source_key; }
    const QString &targetKey() const { return m_target_key; }

    void setRouteIndex(int idx) { m_route_index = idx; }

    void setWaypoints(const QVector<QPointF> &pts) {
        m_waypoints = pts;
        QPainterPath p;
        if (!pts.isEmpty()) {
            p.moveTo(pts.first());
            for (int i = 1; i < pts.size(); ++i)
                p.lineTo(pts[i]);
        }
        setPath(p);
    }

    const QVector<QPointF> &waypoints() const { return m_waypoints; }

    QPainterPath shape() const override {
        QPainterPathStroker stroker;
        stroker.setWidth(10.0);
        return stroker.createStroke(path());
    }

    QRectF boundingRect() const override {
        return QGraphicsPathItem::boundingRect().adjusted(-16, -16, 16, 16);
    }

    void setAppState(AppState *s) { m_state = s; }
    void setWidth(int w) { m_width = w; }

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *option, QWidget *widget) override {
        if (option->state & QStyle::State_Selected) {
            QPen sel_pen(QColor(66, 180, 244), 2.0);
            sel_pen.setCosmetic(true);
            painter->setPen(sel_pen);
            painter->drawPath(path());
        } else {
            QGraphicsPathItem::paint(painter, option, widget);
        }

        if (m_width <= 1)
            return;
        const QPainterPath p = path();
        if (p.elementCount() < 2)
            return;
        qreal best_len = 0;
        QPointF a, b;
        for (int i = 1; i < p.elementCount(); ++i) {
            QPointF p0(p.elementAt(i - 1).x, p.elementAt(i - 1).y);
            QPointF p1(p.elementAt(i).x, p.elementAt(i).y);
            qreal dx = p1.x() - p0.x(), dy = p1.y() - p0.y();
            qreal len = std::sqrt(dx * dx + dy * dy);
            if (len > best_len) {
                best_len = len;
                a = p0;
                b = p1;
            }
        }
        if (best_len < 16.0)
            return;
        QPointF mid((a.x() + b.x()) / 2.0, (a.y() + b.y()) / 2.0);
        QPointF dir(b.x() - a.x(), b.y() - a.y());
        qreal dlen = std::sqrt(dir.x() * dir.x() + dir.y() * dir.y());
        QPointF perp(-dir.y() / dlen, dir.x() / dlen);
        constexpr qreal kSlashHalf = 5.0;
        QPointF s1(mid.x() + perp.x() * kSlashHalf - dir.x() / dlen * kSlashHalf,
                   mid.y() + perp.y() * kSlashHalf - dir.y() / dlen * kSlashHalf);
        QPointF s2(mid.x() - perp.x() * kSlashHalf + dir.x() / dlen * kSlashHalf,
                   mid.y() - perp.y() * kSlashHalf + dir.y() / dlen * kSlashHalf);
        QPen slash_pen(colorForNet(m_source_key), 1.5);
        slash_pen.setCosmetic(true);
        painter->setPen(slash_pen);
        painter->drawLine(s1, s2);
        painter->setPen(QColor(220, 220, 220));
        QFont f = painter->font();
        f.setPointSizeF(9.0);
        painter->setFont(f);
        QPointF label_pt(mid.x() + perp.x() * 9.0 - 4.0, mid.y() + perp.y() * 9.0 + 4.0);
        painter->drawText(label_pt, QString::number(m_width));
    }

  protected:
    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override {
        if (!m_state)
            return;
        QMenu menu;
        QAction *renameAct = menu.addAction(QStringLiteral("Rename..."));
        QAction *chosen = menu.exec(event->screenPos());
        if (chosen == renameAct) {
            bool ok = false;
            QString current;
            QString text =
                QInputDialog::getText(nullptr, QStringLiteral("Rename Wire"),
                                      QStringLiteral("Alias for %1:").arg(m_source_key),
                                      QLineEdit::Normal, current, &ok);
            if (ok) {
                m_state->set_alias(m_source_key, text.trimmed());
            }
        }
        event->accept();
    }

  private:
    QString m_source_key;
    QString m_target_key;
    AppState *m_state = nullptr;
    int m_width = 1;
    int m_route_index = -1;
    QVector<QPointF> m_waypoints;
};

} // namespace hdlc
