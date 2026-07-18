// GUI scene item classes placed in the canvas QGraphicsScene.
//
// All graphics-item class declarations live here under namespace hdlc.
// Method bodies that depend on WireTool, CanvasLayer, or app-level dialog
// helpers are declared here and defined out-of-line in app.cpp (wrapped in
// `namespace hdlc { ... }`) until canvas.h/.cpp is extracted.

#pragma once

#include "canvas_constants.h"

#include <QColor>
#include <QFont>
#include <QGraphicsEllipseItem>
#include <QGraphicsItem>
#include <QGraphicsPathItem>
#include <QGraphicsRectItem>
#include <QGraphicsScene>
#include <QGraphicsSceneContextMenuEvent>
#include <QGraphicsSceneHoverEvent>
#include <QGraphicsSceneMouseEvent>
#include <QHash>
#include <QInputDialog>
#include <QLineEdit>
#include <QMenu>
#include <QPainter>
#include <QPainterPath>
#include <QPainterPathStroker>
#include <QPen>
#include <QPointF>
#include <QPolygonF>
#include <QSet>
#include <QString>
#include <QStringLiteral>
#include <QStyle>
#include <QStyleOptionGraphicsItem>
#include <QTimer>
#include <QVector>
#include <cmath>
#include <vector>

#include "hdl-compose/src/gui/bridge.cxxqt.h"

namespace hdlc {

class WireTool;
class CanvasLayer;

enum class PinSide { Left, Right };

class InstanceItem;
class TopPortItem;
class PortPinItem;

// Width annotation rendered next to multi-bit pins ("[N-1:0]").
inline QString format_width(int w) {
    if (w <= 0)
        return QString();
    return QStringLiteral("[%1:0]").arg(w - 1);
}

// Find the index of an instance by name in the bridge's instance list.
// Definition lives in app.cpp inside `namespace hdlc { ... }`.
int find_instance_index(AppState *state, const QString &name);

// --- NetKey ------------------------------------------------------------------
//
// Codec for the string keys the canvas exchanges with the bridge:
//   "<inst>.<port>"   instance pin
//   "top:<name>"      top-level port
//   either form may carry a "[h]" / "[h:l]" slice suffix
// All construction and parsing of these strings lives here.
struct NetKey {
    QString instance; // empty for top-level ports
    QString port;     // port (or top-port) name, slice suffix stripped
    bool is_top = false;
    bool valid = false;

    static QString topPrefix() { return QStringLiteral("top:"); }
    static QString forTop(const QString &name) { return topPrefix() + name; }
    static QString forPin(const QString &inst, const QString &port) {
        return QStringLiteral("%1.%2").arg(inst, port);
    }
    // Strip a trailing "[...]" slice suffix.
    static QString base(const QString &key) {
        int b = key.indexOf(QChar('['));
        return (b >= 0) ? key.left(b) : key;
    }
    static NetKey parse(const QString &key) {
        NetKey k;
        QString s = base(key);
        if (s.startsWith(topPrefix())) {
            k.is_top = true;
            k.port = s.mid(4);
            k.valid = !k.port.isEmpty();
            return k;
        }
        int dot = s.indexOf(QChar('.'));
        if (dot <= 0 || dot == s.size() - 1)
            return k;
        k.instance = s.left(dot);
        k.port = s.mid(dot + 1);
        k.valid = true;
        return k;
    }
};

// --- JunctionDotItem --------------------------------------------------------

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

// --- WireItem ---------------------------------------------------------------

class WireItem : public QGraphicsPathItem {
  public:
    WireItem(const QString &source_key, const QString &target_key)
        : m_source_key(source_key), m_target_key(target_key) {
        setFlag(QGraphicsItem::ItemIsSelectable, true);
        setAcceptedMouseButtons(Qt::LeftButton | Qt::RightButton);
        setAcceptHoverEvents(true);
        QPen pen(colorForNet(source_key), 1.5);
        pen.setCosmetic(true);
        setPen(pen);
    }

    static QColor colorForNet(const QString &key) {
        // FNV-1a instead of qHash: qHash is seeded per-process, which made
        // net colors change on every app launch.
        quint32 h = 2166136261u;
        for (QChar c : key) {
            h ^= c.unicode();
            h *= 16777619u;
        }
        int hue = static_cast<int>(h % 360);
        return QColor::fromHsv(hue, 110, 200);
    }

    const QString &sourceKey() const { return m_source_key; }
    const QString &targetKey() const { return m_target_key; }

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
        // Trim the first and last segments out of the hit shape so the wire
        // doesn't intercept clicks on the port pin under its endpoints. The
        // wire still RENDERS edge-to-edge — only its clickable region is
        // shortened to the trunk/bridge segments between the stubs.
        if (m_waypoints.size() >= 4) {
            QPainterPath trimmed;
            trimmed.moveTo(m_waypoints[1]);
            for (int i = 2; i < m_waypoints.size() - 1; ++i)
                trimmed.lineTo(m_waypoints[i]);
            return stroker.createStroke(trimmed);
        }
        return stroker.createStroke(path());
    }

    QRectF boundingRect() const override {
        return QGraphicsPathItem::boundingRect().adjusted(-16, -16, 16, 16);
    }

    void setAppState(AppState *s) { m_state = s; }
    void setWidth(int w) {
        m_width = w;
        refreshPen();
    }
    void setCanvasLayer(CanvasLayer *layer) { m_layer = layer; }

    // Net hover: brighter, thicker pen on every wire of the hovered net so
    // fan-out is traceable. Driven by CanvasLayer::setHoveredNet.
    void setNetHover(bool hover) {
        if (m_net_hover == hover)
            return;
        m_net_hover = hover;
        refreshPen();
    }

    // Base weight scales with bus width so buses read heavier than scalars.
    void refreshPen() {
        QColor c = colorForNet(m_source_key);
        qreal base = m_width > 1 ? 2.2 : 1.5;
        if (m_net_hover) {
            c = QColor::fromHsv(c.hue(), 160, 255);
            base += 1.0;
        }
        QPen p(c, base);
        p.setCosmetic(true);
        setPen(p);
    }

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
        // Prefer the driver stub (first segment) for the width marker: each
        // pin has its own Y, so markers can't stack the way they did on the
        // longest segment — usually a gutter lane 12 px from its neighbours.
        qreal best_len = 0;
        QPointF a, b;
        {
            QPointF p0(p.elementAt(0).x, p.elementAt(0).y);
            QPointF p1(p.elementAt(1).x, p.elementAt(1).y);
            qreal dx = p1.x() - p0.x(), dy = p1.y() - p0.y();
            qreal len = std::sqrt(dx * dx + dy * dy);
            if (len >= 24.0) {
                best_len = len;
                a = p0;
                b = p1;
            }
        }
        if (best_len == 0) {
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
        QPen slash_pen(pen().color(), pen().widthF());
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
    // Defined in canvas.cpp — they need the full CanvasLayer type.
    void hoverEnterEvent(QGraphicsSceneHoverEvent *event) override;
    void hoverLeaveEvent(QGraphicsSceneHoverEvent *event) override;

    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override {
        if (!m_state)
            return;
        QMenu menu;
        QAction *aliasAct = menu.addAction(QStringLiteral("Set Net Alias..."));
        QAction *chosen = menu.exec(event->screenPos());
        if (chosen == aliasAct) {
            bool ok = false;
            QString current = m_state->net_alias(m_source_key);
            QString text =
                QInputDialog::getText(nullptr, QStringLiteral("Net Alias"),
                                      QStringLiteral("Signal name for %1 (empty resets to default):")
                                          .arg(m_source_key),
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
    CanvasLayer *m_layer = nullptr;
    int m_width = 1;
    bool m_net_hover = false;
    QVector<QPointF> m_waypoints;
};

// --- PortPinItem ------------------------------------------------------------

class PortPinItem : public QGraphicsItem {
  public:
    PortPinItem(const QString &name, int direction, int width, PinSide side, InstanceItem *parent);
    ~PortPinItem() override;

    void setSlot(int slot_index) {
        m_slot = slot_index;
        prepareGeometryChange();
        update();
    }

    void setKey(const QString &k) { m_key = k; }
    QString key() const { return m_key; }
    int direction() const { return m_direction; }
    int width() const { return m_width; }
    QString portName() const { return m_name; }
    PinSide side() const { return m_side; }

    void setWireTool(WireTool *wt) { m_wire_tool = wt; }
    void setArmedState(bool armed) {
        if (m_armed_state != armed) {
            m_armed_state = armed;
            update();
        }
    }
    bool armedState() const { return m_armed_state; }

    virtual QPointF tipScenePos() const;

    void flashRed(int ms = 500);

    QRectF boundingRect() const override;
    QPainterPath shape() const override;
    void paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) override;

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseMoveEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseReleaseEvent(QGraphicsSceneMouseEvent *event) override;
    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override;

    QString m_name;
    QString m_key;
    int m_direction;
    int m_width;
    PinSide m_side;
    InstanceItem *m_parent;
    int m_slot = 0;
    WireTool *m_wire_tool = nullptr;
    bool m_flash = false;
    bool m_armed_state = false;
};

// --- BundlePinItem ---------------------------------------------------------

class BundlePinItem : public PortPinItem {
  public:
    BundlePinItem(const QString &bundle_name, PinSide side, bool header, int member_count, InstanceItem *parent)
        : PortPinItem(bundle_name, -1, 0, side, parent), m_header(header), m_member_count(member_count) {
        setAcceptedMouseButtons(Qt::LeftButton | Qt::RightButton);
    }

    QRectF boundingRect() const override;
    QPainterPath shape() const override;
    QPointF tipScenePos() const override;
    void paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) override;

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseMoveEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseReleaseEvent(QGraphicsSceneMouseEvent *event) override;

  private:
    bool m_header;
    int m_member_count;
    QPointF m_press_scene;
    QPointF m_parent_start;
    bool m_dragged = false;
};

// --- GroupHullItem ----------------------------------------------------------

class InstanceItem;

/// The "bubble" around an expanded group's members: translucent hull with
/// the group name and a '−' button that collapses the group down to its
/// boundary ports. Purely visual — only the title strip and button take
/// clicks, so wiring and rubber-band selection work through the interior.
class GroupHullItem : public QGraphicsItem {
  public:
    GroupHullItem(AppState *state, const QString &group)
        : m_state(state), m_group(group) {
        setZValue(-5); // behind wires, instances, pins
        setAcceptedMouseButtons(Qt::LeftButton);
    }

    void setMembers(const QVector<QGraphicsItem *> &members) {
        m_members = members;
        refreshGeometry();
    }

    // Track member drags: hull follows the union of member rects.
    void refreshGeometry() {
        QRectF r;
        for (auto *it : m_members) {
            QRectF b = it->sceneBoundingRect();
            r = r.isNull() ? b : r.united(b);
        }
        r.adjust(-kGroupHullPadding, -kGroupHullPadding - kGroupHullTitleHeight,
                 kGroupHullPadding, kGroupHullPadding);
        if (r != m_rect) {
            prepareGeometryChange();
            m_rect = r;
        }
        update();
    }

    QRectF boundingRect() const override { return m_rect; }

    QPainterPath shape() const override {
        QPainterPath p;
        p.addRect(glyphRect());
        p.addRect(titleRect());
        return p;
    }

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) override {
        if (m_rect.isNull())
            return;
        painter->setRenderHint(QPainter::Antialiasing);
        QPen border(QColor(170, 140, 220, 140), 1.4, Qt::DashLine);
        border.setCosmetic(true);
        painter->setPen(border);
        painter->setBrush(QColor(170, 140, 220, 14));
        painter->drawRoundedRect(m_rect, 10, 10);

        QFont f = painter->font();
        f.setBold(true);
        painter->setFont(f);
        painter->setPen(QColor(196, 176, 232));
        painter->drawText(titleRect(), Qt::AlignLeft | Qt::AlignVCenter,
                          painter->fontMetrics().elidedText(m_group, Qt::ElideRight, 200));

        QRectF g = glyphRect();
        painter->setPen(QPen(QColor(170, 140, 220, 180), 1.0));
        painter->setBrush(QColor(46, 42, 58));
        painter->drawRoundedRect(g, 4, 4);
        painter->setPen(QColor(226, 228, 232));
        painter->drawText(g, Qt::AlignCenter, QStringLiteral("−"));
    }

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override {
        if (event->button() == Qt::LeftButton && glyphRect().contains(event->pos())) {
            // Collapse rebuilds the canvas and deletes THIS item — defer.
            AppState *st = m_state;
            const QString grp = m_group;
            QTimer::singleShot(0, [st, grp]() { st->set_group_collapsed(grp, true); });
            event->accept();
            return;
        }
        event->ignore();
    }

  private:
    QRectF glyphRect() const {
        return QRectF(m_rect.right() - 26, m_rect.top() + 5, 18, 18);
    }
    QRectF titleRect() const {
        return QRectF(m_rect.left() + 10, m_rect.top() + 2, 210, kGroupHullTitleHeight);
    }

    AppState *m_state;
    QString m_group;
    QVector<QGraphicsItem *> m_members;
    QRectF m_rect;
};

// --- InstanceItem -----------------------------------------------------------

class InstanceItem : public QGraphicsRectItem {
  public:
    InstanceItem(AppState *state, const QString &name, const QString &module, QGraphicsItem *parent = nullptr)
        : QGraphicsRectItem(0, 0, kMinInstanceWidth, kInstanceHeaderHeight + kMinInstanceBodyHeight, parent),
          m_state(state), m_name(name), m_module(module) {
        setFlags(QGraphicsItem::ItemIsMovable | QGraphicsItem::ItemIsSelectable |
                 QGraphicsItem::ItemSendsGeometryChanges);
        setAcceptedMouseButtons(Qt::LeftButton);
        layoutPins();
    }

    QString instanceName() const { return m_name; }
    AppState *state() const { return m_state; }
    void setInstanceName(const QString &n) { m_name = n; }
    void setModuleRef(const QString &m) { m_module = m; }
    int width() const { return m_width; }

    void setWireTool(WireTool *wt) {
        m_wire_tool = wt;
        for (auto *pin : m_pins) {
            pin->setWireTool(wt);
        }
    }

    void setCanvasLayer(CanvasLayer *layer) { m_canvas_layer = layer; }

    QPointF portAnchorScenePos(const QString &port_name) const {
        auto it = m_port_anchor.find(port_name);
        if (it == m_port_anchor.end()) {
            return mapToScene(rect().center());
        }
        return it.value()->tipScenePos();
    }

    // The pin/header item a port resolves to. Collapsed bundle members all
    // return the same header item; expanded members return their own pins.
    // Used to merge a collapsed bundle's wires into one bus by identity.
    const void *portAnchorItem(const QString &port_name) const {
        auto it = m_port_anchor.find(port_name);
        return (it == m_port_anchor.end()) ? nullptr : static_cast<const void *>(it.value());
    }

    void toggleBundleExpanded(const QString &bundle);

    bool bundleExpanded(const QString &bundle) const { return m_expanded_bundles.contains(bundle); }

    // Max pixel width a pin label may occupy on the given side before
    // eliding. Computed in layoutPins; prevents left/right labels from
    // overlapping now that instance width is capped at kMaxInstanceWidth.
    int labelBudget(PinSide side) const {
        return (side == PinSide::Left) ? m_left_label_budget : m_right_label_budget;
    }

    void relayoutPins() {
        for (auto *pin : m_pins) {
            scene()->removeItem(pin);
            delete pin;
        }
        m_pins.clear();
        layoutPins();
        update();
    }

    // End-of-drag: settle clear of other modules (deferred from the live drag),
    // then persist the position. Defined in items.cpp (needs CanvasLayer).
    void commitDragPosition();

    void layoutPins();

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *option, QWidget *) override;

    // The +/- group-toggle button in the header's top-right corner.
    QRectF toggleGlyphRect() const {
        return QRectF(rect().right() - 26, rect().top() + 7, 18, 18);
    }

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override {
        if (event->button() == Qt::LeftButton && m_state && m_state->is_group_block(m_name)
            && toggleGlyphRect().contains(event->pos())) {
            // Expansion rebuilds the canvas and deletes THIS item — defer
            // past the event handler, capture by value.
            AppState *st = m_state;
            const QString grp = m_name;
            QTimer::singleShot(0, [st, grp]() { st->set_group_collapsed(grp, false); });
            event->accept();
            return;
        }
        if (event->button() == Qt::LeftButton) {
            m_pressScenePos = event->scenePos();
            m_dragged = false;
        }
        QGraphicsRectItem::mousePressEvent(event);
    }

    void mouseMoveEvent(QGraphicsSceneMouseEvent *event) override {
        QGraphicsRectItem::mouseMoveEvent(event);
        if (!m_dragged) {
            QPointF delta = event->scenePos() - m_pressScenePos;
            if (delta.manhattanLength() >= kClickThresholdPx) {
                m_dragged = true;
            }
        }
    }

    void mouseReleaseEvent(QGraphicsSceneMouseEvent *event) override {
        QGraphicsRectItem::mouseReleaseEvent(event);
        if (event->button() != Qt::LeftButton) {
            return;
        }
        if (m_dragged) {
            commitDragPosition();
        } else {
            m_state->set_selected_instance(m_name);
        }
        m_dragged = false;
    }

    QVariant itemChange(GraphicsItemChange change, const QVariant &value) override;
    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override;

  private:
    AppState *m_state;
    QString m_name;
    QString m_module;
    QPointF m_pressScenePos;
    bool m_dragged = false;
    std::vector<PortPinItem *> m_pins;
    QSet<QString> m_expanded_bundles;
    QHash<QString, PortPinItem *> m_port_anchor;
    WireTool *m_wire_tool = nullptr;
    CanvasLayer *m_canvas_layer = nullptr;
    int m_width = kMinInstanceWidth;
    int m_left_label_budget = kMinInstanceWidth;
    int m_right_label_budget = kMinInstanceWidth;
};

// --- TopPortItem ------------------------------------------------------------

class TopPortItem : public PortPinItem {
  public:
    TopPortItem(const QString &name, int direction, int width, PinSide side)
        : PortPinItem(name, direction, width, side, /*parent*/ nullptr) {
        setKey(NetKey::forTop(name));
        setAcceptedMouseButtons(Qt::LeftButton);
        // Selectable so rubber-band select + Delete can remove top ports.
        setFlag(QGraphicsItem::ItemIsSelectable, true);
    }

    void setLayer(CanvasLayer *layer) { m_layer = layer; }
    // The edge X this port snaps to; dragging only changes Y.
    void setLockedX(qreal x) { m_locked_x = x; }
    qreal lockedX() const { return m_locked_x; }

    QPointF tipScenePos() const override { return mapToScene(QPointF(0, 0)); }

    QPainterPath shape() const override {
        QPainterPath p;
        qreal half = 9.0;
        p.addRect(QRectF(-half, -half, 2 * half, 2 * half));
        return p;
    }

    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override { event->accept(); }

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *option, QWidget *) override {
        painter->setRenderHint(QPainter::Antialiasing);

        if (option && (option->state & QStyle::State_Selected)) {
            painter->setBrush(Qt::NoBrush);
            painter->setPen(QPen(QColor(66, 180, 244), 2.0));
            painter->drawEllipse(QPointF(0, 0), kPinShapeSize + 2, kPinShapeSize + 2);
        }

        // Validation ring + tooltip, same scheme as PortPinItem::paint.
        paintIssueRing(painter, QPointF(0, 0));

        if (armedState()) {
            painter->setBrush(Qt::NoBrush);
            painter->setPen(QPen(QColor(255, 215, 64), 2.5));
            painter->drawEllipse(QPointF(0, 0), kPinShapeSize + 2, kPinShapeSize + 2);
        }

        QColor c = (direction() == 0) ? QColor(140, 200, 140) : QColor(200, 160, 110);
        painter->setBrush(c);
        painter->setPen(QPen(QColor(30, 30, 30), 1));
        QPolygonF poly;
        if (side() == PinSide::Left) {
            poly << QPointF(-kPinShapeSize, -kPinShapeSize / 2.0) << QPointF(0, 0)
                 << QPointF(-kPinShapeSize, kPinShapeSize / 2.0);
        } else {
            poly << QPointF(0, -kPinShapeSize / 2.0) << QPointF(kPinShapeSize, 0) << QPointF(0, kPinShapeSize / 2.0);
        }
        painter->drawPolygon(poly);

        QString label = portName();
        QString w = format_width(width());
        if (!w.isEmpty())
            label += w;
        QFont f = painter->font();
        f.setBold(true);
        painter->setFont(f);
        painter->setPen(QColor(220, 220, 220));
        label = painter->fontMetrics().elidedText(label, Qt::ElideMiddle, 180);
        if (side() == PinSide::Left) {
            painter->drawText(QRectF(-180 - kPinShapeSize - 6, -kPinSlotHeight / 2.0, 180, kPinSlotHeight),
                              Qt::AlignRight | Qt::AlignVCenter, label);
        } else {
            painter->drawText(QRectF(kPinShapeSize + 6, -kPinSlotHeight / 2.0, 180, kPinSlotHeight),
                              Qt::AlignLeft | Qt::AlignVCenter, label);
        }
    }

    QRectF boundingRect() const override {
        if (side() == PinSide::Left) {
            return QRectF(-180 - kPinShapeSize - 6, -kPinSlotHeight / 2.0, 180 + kPinShapeSize + 6, kPinSlotHeight);
        }
        return QRectF(0, -kPinSlotHeight / 2.0, kPinShapeSize + 6 + 180, kPinSlotHeight);
    }

  private:
    // Defined in items.cpp — needs the full CanvasLayer type for state().
    void paintIssueRing(QPainter *painter, const QPointF &center);

  public:

  protected:
    // Click arms/commits a wire (base behavior); drag repositions the port
    // vertically along its edge. Defined in canvas.cpp (needs CanvasLayer).
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseMoveEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseReleaseEvent(QGraphicsSceneMouseEvent *event) override;

  private:
    CanvasLayer *m_layer = nullptr;
    qreal m_locked_x = 0;
    QPointF m_press_scene;
    qreal m_start_y = 0;
    bool m_moved = false;
};

} // namespace hdlc
