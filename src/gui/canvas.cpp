// Out-of-line method bodies for WireTool + CanvasView::dropEvent.

#include "canvas.h"

#include <QAction>
#include <QCursor>
#include <QGraphicsItem>
#include <QGraphicsPathItem>
#include <QGraphicsSceneMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPen>
#include <QStringList>
#include <QStringLiteral>
#include <QToolTip>
#include <cmath>

namespace hdlc {

namespace {

QString allocate_instance_name(AppState *state, const QString &module) {
    int count = state->instance_count();
    QStringList existing;
    for (int i = 0; i < count; ++i) {
        existing << state->instance_name(i);
    }
    for (int i = 0;; ++i) {
        QString candidate = QStringLiteral("%1_%2").arg(module).arg(i);
        if (!existing.contains(candidate)) {
            return candidate;
        }
    }
}

bool parse_pin_key(PortPinItem *pin, QString *inst, QString *port) {
    NetKey k = NetKey::parse(pin->key());
    if (!k.valid || k.is_top)
        return false;
    *inst = k.instance;
    *port = k.port;
    return true;
}

} // namespace

// --- WireItem hover (needs full CanvasLayer type) ----------------------------

void WireItem::hoverEnterEvent(QGraphicsSceneHoverEvent *) {
    if (m_layer)
        m_layer->setHoveredNet(CanvasLayer::baseKey(m_source_key));
}

void WireItem::hoverLeaveEvent(QGraphicsSceneHoverEvent *) {
    if (m_layer)
        m_layer->setHoveredNet(QString());
}

// --- WireTool ---------------------------------------------------------------

QString WireTool::compatibilityError(PortPinItem *src, PortPinItem *dst) const {
    if (src == dst) {
        return QStringLiteral("cannot connect pin to itself");
    }
    // Bundle headers have no key/direction. Without this guard the driver
    // fallback below picked the bundle as driver with an empty RHS, which
    // silently cleared the other pin's connection.
    if (src->key().isEmpty() || dst->key().isEmpty()) {
        return QStringLiteral("cannot wire a bundle header; expand it and wire a member port");
    }
    bool src_top = NetKey::parse(src->key()).is_top;
    bool dst_top = NetKey::parse(dst->key()).is_top;
    if (!src_top && !dst_top && src->direction() == 1 && dst->direction() == 1) {
        return QStringLiteral("output-to-output: only one driver per net allowed");
    }
    int sw = src->width();
    int dw = dst->width();
    if (sw == 1)
        sw = 0;
    if (dw == 1)
        dw = 0;
    if (sw == 0 && dw == -1) {
        return QStringLiteral("type mismatch: scalar cannot drive vector");
    }
    if (sw == -1 && dw == 0) {
        return QStringLiteral("type mismatch: vector cannot drive scalar");
    }
    if (sw >= 0 && dw >= 0 && sw != dw) {
        return QStringLiteral("width mismatch: %1 vs %2")
            .arg(sw == 0 ? QStringLiteral("scalar") : QString::number(sw))
            .arg(dw == 0 ? QStringLiteral("scalar") : QString::number(dw));
    }
    return QString();
}

void WireTool::cancel() {
    clearProvisional();
    if (m_armed) {
        PortPinItem *prev = m_armed;
        m_armed = nullptr;
        prev->setArmedState(false);
    }
}

void WireTool::clearProvisional() {
    if (m_provisional) {
        m_scene->removeItem(m_provisional);
        delete m_provisional;
        m_provisional = nullptr;
    }
}

void WireTool::createProvisional(PortPinItem *from, const QPointF &scene_pos) {
    clearProvisional();
    m_provisional = new QGraphicsPathItem();
    QPen pen(QColor(255, 215, 64, 180), 1.5);
    pen.setCosmetic(true);
    pen.setStyle(Qt::DashLine);
    m_provisional->setPen(pen);
    m_provisional->setZValue(1000);
    // The line ends at the cursor; it must never swallow the closing click.
    m_provisional->setAcceptedMouseButtons(Qt::NoButton);
    QPainterPath p;
    p.moveTo(from->tipScenePos());
    p.lineTo(scene_pos);
    m_provisional->setPath(p);
    m_scene->addItem(m_provisional);
}

bool WireTool::tryCommit(PortPinItem *src, PortPinItem *dst) {
    QString err = compatibilityError(src, dst);
    if (!err.isEmpty()) {
        dst->flashRed();
        QToolTip::showText(QCursor::pos(), err);
        return false;
    }
    NetKey src_k = NetKey::parse(src->key());
    NetKey dst_k = NetKey::parse(dst->key());

    if (src_k.is_top && dst_k.is_top) {
        QToolTip::showText(QCursor::pos(), QStringLiteral("cannot wire two top-level ports"));
        return false;
    }

    if (src_k.is_top || dst_k.is_top) {
        const NetKey &top = src_k.is_top ? src_k : dst_k;
        const NetKey &inst_pin = src_k.is_top ? dst_k : src_k;
        if (!inst_pin.valid)
            return false;
        m_state->set_port_map_entry(inst_pin.instance, inst_pin.port, top.port);
        return true;
    }

    if (src->direction() == 0 && dst->direction() == 0) {
        return tryCommitMultiLoad(src, dst);
    }
    auto can_drive = [](int dir) { return dir == 1 || dir == 2; };
    PortPinItem *driver = can_drive(src->direction()) ? src : dst;
    const NetKey &load_k = (driver == src) ? dst_k : src_k;
    const NetKey &driver_k = (driver == src) ? src_k : dst_k;
    PortPinItem *load = (driver == src) ? dst : src;
    if (!load_k.valid)
        return false;
    if (driver->width() == 1 && load->width() == 0 && driver_k.valid && !driver_k.is_top) {
        m_state->set_port_map_entry_slice(load_k.instance, load_k.port, driver_k.instance,
                                          driver_k.port, 0, 0);
        return true;
    }
    QString driver_rhs =
        driver_k.is_top ? driver_k.port : NetKey::forPin(driver_k.instance, driver_k.port);
    m_state->set_port_map_entry(load_k.instance, load_k.port, driver_rhs);
    return true;
}

bool WireTool::tryCommitMultiLoad(PortPinItem *a, PortPinItem *b) {
    QString a_inst, a_port, b_inst, b_port;
    if (!parse_pin_key(a, &a_inst, &a_port))
        return false;
    if (!parse_pin_key(b, &b_inst, &b_port))
        return false;

    QString a_rhs = m_state->port_map_entry(a_inst, a_port);
    QString b_rhs = m_state->port_map_entry(b_inst, b_port);
    bool a_driven = !a_rhs.isEmpty();
    bool b_driven = !b_rhs.isEmpty();

    if (a_driven != b_driven) {
        const QString &rhs = a_driven ? a_rhs : b_rhs;
        const QString &target_inst = a_driven ? b_inst : a_inst;
        const QString &target_port = a_driven ? b_port : a_port;
        return m_state->set_port_map_entry(target_inst, target_port, rhs);
    }
    if (a_driven && b_driven && a_rhs == b_rhs) {
        return true;
    }
    QString rhs = NetKey::forPin(a_inst, a_port);
    m_state->set_port_map_entry(a_inst, a_port, rhs);
    m_state->set_port_map_entry(b_inst, b_port, rhs);
    m_sticky_after_commit = true;
    return true;
}

void WireTool::onPinPressed(PortPinItem *pin, const QPointF &scene_pos) {
    if (!pin)
        return;
    m_press_pos = scene_pos;
    if (m_armed && m_armed != pin) {
        tryCommit(m_armed, pin);
        if (m_sticky_after_commit) {
            m_sticky_after_commit = false;
            clearProvisional();
            return;
        }
        cancel();
        return;
    }
    if (m_armed == pin) {
        cancel();
        return;
    }
    m_armed = pin;
    pin->setArmedState(true);
    createProvisional(pin, scene_pos);
}

void WireTool::onPinDragMove(const QPointF &scene_pos) {
    if (!m_armed || !m_provisional)
        return;
    QPainterPath p;
    p.moveTo(m_armed->tipScenePos());
    p.lineTo(scene_pos);
    m_provisional->setPath(p);
}

void WireTool::onPinReleased(const QPointF &scene_pos) {
    if (!m_armed)
        return;
    PortPinItem *target = nullptr;
    for (QGraphicsItem *it : m_scene->items(scene_pos)) {
        if (auto *pin = dynamic_cast<PortPinItem *>(it)) {
            // Skip bundle headers (empty key) — releasing over one must not
            // commit a wire against it.
            if (pin != m_armed && !pin->key().isEmpty()) {
                target = pin;
                break;
            }
        }
    }
    if (target) {
        tryCommit(m_armed, target);
        if (m_sticky_after_commit) {
            m_sticky_after_commit = false;
            clearProvisional();
            return;
        }
        cancel();
        return;
    }
    if ((scene_pos - m_press_pos).manhattanLength() < kClickThresholdPx) {
        // Click-mode arm: keep the provisional line; CanvasView mouse moves
        // track it to the cursor until the closing click or cancel.
        return;
    }
    cancel();
}

// --- CanvasView -------------------------------------------------------------

void CanvasView::dropEvent(QDropEvent *event) {
    QByteArray data = event->mimeData()->data(QString::fromLatin1(kModuleMimeType));
    if (data.isEmpty()) {
        return;
    }
    QString module = QString::fromUtf8(data);
    QString inst_name = allocate_instance_name(m_state, module);
    if (!m_state->add_instance(inst_name, module)) {
        return;
    }
    QPointF scene_pos = mapToScene(event->position().toPoint());
    int col = static_cast<int>(std::round(scene_pos.x() / kColumnPitch));
    qreal w = kMinInstanceWidth;
    qreal y = scene_pos.y();
    InstanceItem *item = m_canvas_layer ? m_canvas_layer->itemFor(inst_name) : nullptr;
    if (item)
        w = item->width();
    qreal snapped_x = col * kColumnPitch - w / 2.0;
    // Resolve overlap here so the MODEL gets the corrected position — the
    // itemChange hook only corrects the visual item, and an uncorrected
    // model position reloads as overlapping modules after save.
    if (item && m_canvas_layer)
        y = m_canvas_layer->resolveClearY(item, snapped_x, w, item->rect().height(), y);
    m_state->set_instance_position(inst_name, snapped_x, y);
    event->acceptProposedAction();
}

} // namespace hdlc
