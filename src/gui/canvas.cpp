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

// --- TopPortItem drag (needs full CanvasLayer type) -------------------------

void TopPortItem::mousePressEvent(QGraphicsSceneMouseEvent *event) {
    // Shift/Ctrl/Cmd + click toggles this port in the selection (append or
    // remove) instead of arming a wire — the plain click is taken by wiring,
    // so modifier-click is how to multi-select for Delete / group drags.
    if (event->button() == Qt::LeftButton
        && (event->modifiers() & (Qt::ShiftModifier | Qt::ControlModifier))) {
        setSelected(!isSelected());
        event->accept();
        return;
    }
    if (event->button() == Qt::LeftButton) {
        m_press_scene = event->scenePos();
        m_start_y = pos().y();
        m_moved = false;
        // A drag starting on a selected port moves every selected top port
        // together, keeping their relative spacing.
        m_drag_group.clear();
        if (isSelected() && scene()) {
            for (QGraphicsItem *it : scene()->selectedItems()) {
                if (auto *tp = dynamic_cast<TopPortItem *>(it)) {
                    if (tp != this)
                        m_drag_group.append({tp, tp->pos().y()});
                }
            }
        }
        event->accept();
        return;
    }
    PortPinItem::mousePressEvent(event);
}

void TopPortItem::mouseMoveEvent(QGraphicsSceneMouseEvent *event) {
    if ((event->buttons() & Qt::LeftButton) && m_layer) {
        if (!m_moved && (event->scenePos() - m_press_scene).manhattanLength() >= kClickThresholdPx)
            m_moved = true;
        if (m_moved) {
            const qreal dy = event->scenePos().y() - m_press_scene.y();
            qreal y = m_layer->clampTopPortY(m_start_y + dy);
            setPos(m_locked_x, y); // X stays pinned to the edge
            for (const auto &entry : m_drag_group) {
                TopPortItem *tp = entry.first;
                tp->setPos(tp->lockedX(), m_layer->clampTopPortY(entry.second + dy));
            }
            m_layer->replanWires();
        }
        event->accept();
        return;
    }
    PortPinItem::mouseMoveEvent(event);
}

void TopPortItem::mouseReleaseEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton) {
        if (m_moved) {
            m_moved = false;
            if (m_layer) {
                m_layer->setTopPortY(portName(), pos().y());
                for (const auto &entry : m_drag_group)
                    m_layer->setTopPortY(entry.first->portName(), entry.first->pos().y());
            }
        } else if (m_wire_tool) {
            // No drag: treat as a click — arm/commit a wire like a normal pin.
            m_wire_tool->onPinPressed(this, event->scenePos());
        }
        m_drag_group.clear();
        event->accept();
        return;
    }
    PortPinItem::mouseReleaseEvent(event);
}

// --- WireTool ---------------------------------------------------------------

bool WireTool::tryCommit(PortPinItem *src, PortPinItem *dst) {
    // All wiring semantics (compatibility, driver/load resolution,
    // multi-load nets) live Rust-side — see AppState::connect_pins.
    FfiConnectResult r = m_state->connect_pins(src->key(), dst->key());
    if (!r.committed) {
        if (!r.error.empty()) {
            dst->flashRed();
            QToolTip::showText(QCursor::pos(),
                               QString::fromUtf8(r.error.data(), static_cast<int>(r.error.size())));
        }
        return false;
    }
    m_sticky_after_commit = r.sticky;
    return true;
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
    // Batch so add + initial position form a single undo step.
    m_state->begin_batch();
    if (!m_state->add_instance(inst_name, module)) {
        m_state->end_batch();
        return;
    }
    QPointF scene_pos = mapToScene(event->position().toPoint());
    // Resolve placement here so the MODEL gets the corrected position — the
    // itemChange hook only corrects the visual item, and an uncorrected
    // model position reloads as overlapping modules after save.
    InstanceItem *item = m_canvas_layer ? m_canvas_layer->itemFor(inst_name) : nullptr;
    QPointF pos(scene_pos.x() - kMinInstanceWidth / 2.0, scene_pos.y());
    if (item && m_canvas_layer) {
        // Snap X to the drop column, keep the drop Y; push-shove settles the
        // rest of the column (and persists every changed position).
        const int col = static_cast<int>(std::round(scene_pos.x() / kColumnPitch));
        pos = QPointF(col * kColumnPitch - item->width() / 2.0, scene_pos.y());
        item->setPos(pos);
        m_canvas_layer->settleAfterMove({item});
    } else {
        m_state->set_instance_position(inst_name, pos.x(), pos.y());
    }
    m_state->end_batch();
    event->acceptProposedAction();
}

} // namespace hdlc
