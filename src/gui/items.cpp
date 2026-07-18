// Out-of-line method bodies for the scene items declared in items.h:
// PortPinItem, BundlePinItem, InstanceItem (paint, pin layout, placement),
// plus the two small dialogs reachable from a pin's context menu.

#include "canvas.h" // full CanvasLayer/WireTool types for itemChange + the dtor

#include <QCheckBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFont>
#include <QFontMetrics>
#include <QFormLayout>
#include <QInputDialog>
#include <QLabel>
#include <QScrollArea>
#include <QVBoxLayout>
#include <cmath>

namespace hdlc {

PortPinItem::PortPinItem(const QString &name, int direction, int width, PinSide side, InstanceItem *parent)
    : QGraphicsItem(parent), m_name(name), m_direction(direction), m_width(width), m_side(side), m_parent(parent) {
    setAcceptedMouseButtons(Qt::LeftButton);
    QString tip = m_name;
    QString w = format_width(m_width);
    if (!w.isEmpty())
        tip += w;
    setToolTip(tip);
}

QPointF PortPinItem::tipScenePos() const {
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;
    return mapToScene(QPointF(tip_x, y));
}

QRectF PortPinItem::boundingRect() const {
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal w = pw + 40;
    // Left margin covers the exposed-signal diamond drawn outside the tip.
    qreal x = (m_side == PinSide::Left) ? -kPinShapeSize - 14 : 0;
    // The armed-state halo (drawn at the pin tip with radius kPinShapeSize+2)
    // extends a few pixels above/below the slot rect. Without this Y margin,
    // those pixels fall outside boundingRect and Qt skips repainting them on
    // deselection, leaving a stale halo arc visible.
    constexpr qreal kHaloMarginY = 6.0;
    return QRectF(x, m_slot * kPinSlotHeight - kHaloMarginY, w, kPinSlotHeight + 2 * kHaloMarginY);
}

QPainterPath PortPinItem::shape() const {
    QPainterPath p;
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;
    qreal half = 9.0;
    p.addRect(QRectF(tip_x - half, y - half, 2 * half, 2 * half));
    return p;
}

QRectF BundlePinItem::boundingRect() const {
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal w = pw + 40;
    qreal x = (m_side == PinSide::Left) ? -kPinShapeSize - 8 : 0;
    return QRectF(x, m_slot * kPinSlotHeight, w, kPinSlotHeight);
}

// The whole row (arrow + label + button) folds the bundle — not just the
// button glyph. Without this the hit target is PortPinItem's tiny tip square.
QPainterPath BundlePinItem::shape() const {
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal half = kPinShapeSize;
    qreal budget = m_parent ? m_parent->labelBudget(m_side) : pw;
    // Confine the hit region to this side's button + label so it can't span the
    // module and swallow clicks on the opposite side's ports.
    qreal left, right;
    if (m_side == PinSide::Left) {
        left = -half;
        right = qMin(pw, half + 4 + budget);
    } else {
        left = qMax(qreal(0), pw - half - 4 - budget);
        right = pw + half;
    }
    QPainterPath p;
    p.addRect(QRectF(left, m_slot * kPinSlotHeight, right - left, kPinSlotHeight));
    return p;
}

// Collapsed: the fat button straddles the module edge, so the inherited tip
// (the edge) lands at the button's center and wires draw over it. Anchor at the
// button's outer face so member wires emerge from the port instead. Expanded
// headers carry no wire — fall back to the edge.
QPointF BundlePinItem::tipScenePos() const {
    if (m_header)
        return PortPinItem::tipScenePos();
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal half = kPinShapeSize;
    qreal tip_x = (m_side == PinSide::Left) ? -half : pw + half;
    return mapToScene(QPointF(tip_x, y));
}

// format_width moved inline to items.h.

void PortPinItem::paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) {
    // Tip point at slot center, on the side edge.
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;

    // Draw shape
    painter->setRenderHint(QPainter::Antialiasing);

    // Validation ring: red = error (multi-driver etc.), amber = warning
    // (undriven / unconnected input). Tooltip carries the message. Pins on
    // nets exposed as a group boundary port get a small violet diamond just
    // outside the tip — that's where the group's external signal originates.
    if (AppState *state = m_parent ? m_parent->state() : nullptr) {
        int issue = state->pin_issue_level(m_key);
        QString base_tip = m_name + format_width(m_width);
        QString tip = base_tip;
        if (issue > 0) {
            painter->setBrush(Qt::NoBrush);
            QPen ring(issue >= 2 ? QColor(232, 72, 60) : QColor(255, 179, 0), 2.0);
            painter->setPen(ring);
            painter->drawEllipse(QPointF(tip_x, y), kPinShapeSize + 3, kPinShapeSize + 3);
            tip = base_tip + QStringLiteral("\n") + state->pin_issue_message(m_key);
        }
        QString exposed = state->pin_exposed_note(m_key);
        if (!exposed.isEmpty()) {
            qreal bx = (m_side == PinSide::Left) ? tip_x - kPinShapeSize - 8
                                                 : tip_x + kPinShapeSize + 8;
            QPolygonF diamond;
            diamond << QPointF(bx - 4, y) << QPointF(bx, y - 4) << QPointF(bx + 4, y)
                    << QPointF(bx, y + 4);
            painter->setPen(QPen(QColor(120, 90, 170), 1.0));
            painter->setBrush(QColor(196, 160, 255));
            painter->drawPolygon(diamond);
            tip += QStringLiteral("\n") + exposed;
        }
        if (tip != toolTip())
            setToolTip(tip);
    }

    // Armed-state halo: yellow ring around the pin tip when WireTool has armed us.
    if (m_armed_state) {
        painter->setBrush(Qt::NoBrush);
        painter->setPen(QPen(QColor(255, 215, 64), 2.5));
        painter->drawEllipse(QPointF(tip_x, y), kPinShapeSize + 2, kPinShapeSize + 2);
    }

    QColor pin_color(210, 210, 210);
    if (m_flash) {
        pin_color = QColor(232, 92, 80);
    } else if (m_direction == 0 /*In*/) {
        pin_color = QColor(120, 196, 152);
    } else if (m_direction == 1 /*Out*/) {
        pin_color = QColor(232, 168, 104);
    } else if (m_direction == 2 /*InOut*/) {
        pin_color = QColor(160, 174, 224);
    }
    painter->setBrush(pin_color);
    QPen pin_outline(QColor(20, 22, 26), 1);
    pin_outline.setCosmetic(true);
    painter->setPen(pin_outline);

    if (m_direction == 2) {
        // Diamond
        QPolygonF diamond;
        qreal half = kPinShapeSize / 2.0;
        diamond << QPointF(tip_x - half, y) << QPointF(tip_x, y - half) << QPointF(tip_x + half, y)
                << QPointF(tip_x, y + half);
        painter->drawPolygon(diamond);
    } else {
        // Triangle always points right (signal flows L→R). Tip is to the
        // right; base is on the inside of the module for inputs, on the
        // outside (extending past the module edge) for outputs.
        QPolygonF tri;
        if (m_side == PinSide::Left) {
            // Input on left edge: tip at module edge (tip_x), base outside-left.
            tri << QPointF(tip_x - kPinShapeSize, y - kPinShapeSize / 2.0) << QPointF(tip_x, y)
                << QPointF(tip_x - kPinShapeSize, y + kPinShapeSize / 2.0);
        } else {
            // Output on right edge: tip extends past module edge, base at edge.
            tri << QPointF(tip_x, y - kPinShapeSize / 2.0) << QPointF(tip_x + kPinShapeSize, y)
                << QPointF(tip_x, y + kPinShapeSize / 2.0);
        }
        painter->drawPolygon(tri);
    }

    // Label
    QString label = m_name;
    QString w = format_width(m_width);
    if (!w.isEmpty()) {
        label += w;
    }
    QFont f = painter->font();
    f.setPointSizeF(f.pointSizeF() - 1.0);
    if (!w.isEmpty()) {
        f.setFamily(QStringLiteral("Menlo"));
    }
    painter->setFont(f);
    painter->setPen(QColor(220, 220, 220));
    // Elide to this side's budget so left and right labels can't overlap in
    // a width-capped module. Tooltip (set in ctor) still shows the full name.
    int budget = m_parent ? m_parent->labelBudget(m_side) : static_cast<int>(pw) - 8;
    label = painter->fontMetrics().elidedText(label, Qt::ElideMiddle, budget);
    QRectF label_rect;
    if (m_side == PinSide::Left) {
        label_rect = QRectF(tip_x + 4, y - kPinSlotHeight / 2.0, pw - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignLeft | Qt::AlignVCenter, label);
    } else {
        label_rect = QRectF(tip_x - pw + 4, y - kPinSlotHeight / 2.0, pw - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignRight | Qt::AlignVCenter, label);
    }
}

void BundlePinItem::paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) {
    painter->setRenderHint(QPainter::Antialiasing);
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;
    qreal half = kPinShapeSize;

    QString arrow = m_header ? QStringLiteral("▼") : QStringLiteral("▶");
    QString label = QStringLiteral("%1 %2 (%3)").arg(arrow, m_name).arg(m_member_count);

    if (!m_header) {
        // Collapsed: fat rounded rect on the edge.
        QRectF fat(tip_x - half, y - half, 2 * half, 2 * half);
        painter->setBrush(QColor(132, 162, 212));
        QPen ppen(QColor(20, 22, 26), 1.0);
        ppen.setCosmetic(true);
        painter->setPen(ppen);
        painter->drawRoundedRect(fat, 3, 3);
    } else {
        // Expanded header: thin horizontal bar spanning the side.
        QRectF bar;
        if (m_side == PinSide::Left) {
            bar = QRectF(tip_x, y - 1, 6, 2);
        } else {
            bar = QRectF(tip_x - 6, y - 1, 6, 2);
        }
        painter->setPen(Qt::NoPen);
        painter->setBrush(QColor(132, 162, 212));
        painter->drawRect(bar);
    }

    QFont f = painter->font();
    f.setBold(true);
    f.setPointSizeF(f.pointSizeF() - 1.0);
    painter->setFont(f);
    painter->setPen(QColor(m_header ? 180 : 235, m_header ? 200 : 235, m_header ? 220 : 235));
    if (m_parent)
        label = painter->fontMetrics().elidedText(label, Qt::ElideMiddle, m_parent->labelBudget(m_side));
    QRectF label_rect;
    if (m_side == PinSide::Left) {
        label_rect = QRectF(tip_x + half + 4, y - kPinSlotHeight / 2.0, pw - half - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignLeft | Qt::AlignVCenter, label);
    } else {
        label_rect =
            QRectF(tip_x - pw + 4, y - kPinSlotHeight / 2.0, pw - half - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignRight | Qt::AlignVCenter, label);
    }
}

// The bundle row spans most of the module side, so it must not steal drags
// meant to move the module: a click folds the bundle, a drag moves the parent
// instance (reusing its placement + undo path). We drive the parent directly
// rather than letting the press fall through, so the gesture also works over
// the button that overhangs the module edge.
void BundlePinItem::mousePressEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_parent) {
        m_press_scene = event->scenePos();
        m_parent_start = m_parent->pos();
        m_dragged = false;
        event->accept();
        return;
    }
    PortPinItem::mousePressEvent(event);
}

void BundlePinItem::mouseMoveEvent(QGraphicsSceneMouseEvent *event) {
    if (m_parent && (event->buttons() & Qt::LeftButton)) {
        QPointF delta = event->scenePos() - m_press_scene;
        if (!m_dragged && delta.manhattanLength() >= kClickThresholdPx)
            m_dragged = true;
        if (m_dragged)
            m_parent->setPos(m_parent_start + delta);
        event->accept();
        return;
    }
    PortPinItem::mouseMoveEvent(event);
}

void BundlePinItem::mouseReleaseEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_parent) {
        if (m_dragged) {
            m_dragged = false;
            m_parent->commitDragPosition();
            event->accept();
            return;
        }
        // Click: fold/unfold. relayoutPins() deletes `this`, so toggle last.
        event->accept();
        m_parent->toggleBundleExpanded(m_name);
        return;
    }
    PortPinItem::mouseReleaseEvent(event);
}

void InstanceItem::layoutPins() {
    m_port_anchor.clear();
    int idx = find_instance_index(m_state, m_name);
    if (idx < 0) {
        return;
    }
    int port_count = m_state->instance_port_count(idx);

    // Group ports by (side, bundle).
    // Build side ordered lists. Bundles collect their members.
    struct PortEntry {
        int port_index;
        QString name;
        int direction;
        int width;
        QString bundle;
    };
    // Build a reverse map port_name -> manual_bundle_name for this instance.
    // Manual bundles replace any auto-detected `PortDef.bundle` value.
    QHash<QString, QString> manual_bundle_of;
    {
        int bc = m_state->manual_bundle_count(idx);
        for (int b = 0; b < bc; ++b) {
            QString bname = m_state->manual_bundle_name(idx, b);
            int pc = m_state->manual_bundle_port_count(idx, b);
            for (int pp = 0; pp < pc; ++pp) {
                manual_bundle_of.insert(m_state->manual_bundle_port_name(idx, b, pp), bname);
            }
        }
    }

    std::vector<PortEntry> entries;
    entries.reserve(port_count);
    for (int p = 0; p < port_count; ++p) {
        QString port_name = m_state->instance_port_name(idx, p);
        QString bundle = manual_bundle_of.value(port_name, m_state->instance_port_bundle(idx, p));
        entries.push_back(
            {p, port_name, m_state->instance_port_direction(idx, p), m_state->instance_port_width(idx, p), bundle});
    }

    // Direction → side. InOut → Left (convention).
    auto side_for = [](int dir) { return (dir == 1) ? PinSide::Right : PinSide::Left; };

    // Bundle side = majority direction of members. Ties → Left (convention).
    // Two-pass: first count outs vs non-outs per bundle, then place all
    // members on the chosen side regardless of individual direction.
    QMap<QString, int> bundle_out_count;
    QMap<QString, int> bundle_total;
    for (const auto &e : entries) {
        if (e.bundle.isEmpty())
            continue;
        bundle_total[e.bundle] = bundle_total.value(e.bundle, 0) + 1;
        if (e.direction == 1)
            bundle_out_count[e.bundle] = bundle_out_count.value(e.bundle, 0) + 1;
    }
    auto bundle_side = [&](const QString &bname) {
        int outs = bundle_out_count.value(bname, 0);
        int total = bundle_total.value(bname, 0);
        return (outs * 2 > total) ? PinSide::Right : PinSide::Left;
    };

    // QMap keeps bundles sorted by name for stable layout.
    QMap<QString, std::vector<int>> left_bundles;
    QMap<QString, std::vector<int>> right_bundles;
    std::vector<int> left_ports, right_ports;
    for (const auto &e : entries) {
        if (!e.bundle.isEmpty()) {
            auto side = bundle_side(e.bundle);
            auto &map = (side == PinSide::Left) ? left_bundles : right_bundles;
            map[e.bundle].push_back(e.port_index);
        } else {
            auto side = side_for(e.direction);
            auto &list = (side == PinSide::Left) ? left_ports : right_ports;
            list.push_back(e.port_index);
        }
    }

    int left_slot = 0;
    int right_slot = 0;

    auto add_port_pin = [&](int port_idx, PinSide side, int &slot) {
        const auto &e = entries[port_idx];
        auto *pin = new PortPinItem(e.name, e.direction, e.width, side, this);
        pin->setSlot(slot);
        pin->setPos(0, kInstanceHeaderHeight);
        pin->setKey(NetKey::forPin(m_name, e.name));
        pin->setWireTool(m_wire_tool);
        m_pins.push_back(pin);
        m_port_anchor.insert(e.name, pin);
        slot++;
    };
    auto add_bundle_group = [&](const QString &bname, const std::vector<int> &members, PinSide side, int &slot) {
        bool expanded = bundleExpanded(bname);
        auto *header = new BundlePinItem(bname, side, expanded, static_cast<int>(members.size()), this);
        header->setSlot(slot);
        header->setPos(0, kInstanceHeaderHeight);
        m_pins.push_back(header);
        // Collapsed: all member ports anchor to the bundle header.
        if (!expanded) {
            for (int p : members) {
                m_port_anchor.insert(entries[p].name, header);
            }
        }
        slot++;
        if (expanded) {
            for (int p : members) {
                add_port_pin(p, side, slot);
            }
        }
    };

    for (int p : left_ports)
        add_port_pin(p, PinSide::Left, left_slot);
    for (auto it = left_bundles.constBegin(); it != left_bundles.constEnd(); ++it) {
        add_bundle_group(it.key(), it.value(), PinSide::Left, left_slot);
    }
    for (int p : right_ports)
        add_port_pin(p, PinSide::Right, right_slot);
    for (auto it = right_bundles.constBegin(); it != right_bundles.constEnd(); ++it) {
        add_bundle_group(it.key(), it.value(), PinSide::Right, right_slot);
    }

    int slot_total = (left_slot > right_slot) ? left_slot : right_slot;
    int pin_body = slot_total * kPinSlotHeight + 8;
    int body = (pin_body > kMinInstanceBodyHeight) ? pin_body : kMinInstanceBodyHeight;

    // Width = max label column on each side + gap. Use a slightly-smaller
    // font than default to mirror PortPinItem::paint sizing; bold for the
    // header so the title row's reservation doesn't wrap.
    QFont label_font;
    label_font.setPointSizeF(label_font.pointSizeF() - 1.0);
    QFontMetrics fm(label_font);
    QFont header_font;
    header_font.setBold(true);
    header_font.setPointSizeF(header_font.pointSizeF() + 1.0);
    QFontMetrics hf(header_font);

    auto label_for = [&](const PortEntry &e, bool is_bundle_header, int member_count) {
        if (is_bundle_header) {
            return QStringLiteral("▼ %1 (%2)").arg(e.bundle).arg(member_count);
        }
        QString s = e.name;
        if (e.width > 0) {
            s += QStringLiteral("[%1:0]").arg(e.width - 1);
        }
        return s;
    };
    auto pin_label_w = [&](const PortEntry &e) {
        return fm.horizontalAdvance(label_for(e, false, 0));
    };

    int left_label_w = 0;
    for (int p : left_ports) {
        left_label_w = std::max(left_label_w, pin_label_w(entries[p]));
    }
    for (auto it = left_bundles.constBegin(); it != left_bundles.constEnd(); ++it) {
        int hw = fm.horizontalAdvance(QStringLiteral("▼ %1 (%2)").arg(it.key()).arg(static_cast<int>(it.value().size())));
        left_label_w = std::max(left_label_w, hw);
        if (bundleExpanded(it.key())) {
            for (int p : it.value()) {
                left_label_w = std::max(left_label_w, pin_label_w(entries[p]));
            }
        }
    }
    int right_label_w = 0;
    for (int p : right_ports) {
        right_label_w = std::max(right_label_w, pin_label_w(entries[p]));
    }
    for (auto it = right_bundles.constBegin(); it != right_bundles.constEnd(); ++it) {
        int hw = fm.horizontalAdvance(QStringLiteral("▼ %1 (%2)").arg(it.key()).arg(static_cast<int>(it.value().size())));
        right_label_w = std::max(right_label_w, hw);
        if (bundleExpanded(it.key())) {
            for (int p : it.value()) {
                right_label_w = std::max(right_label_w, pin_label_w(entries[p]));
            }
        }
    }
    int header_w = hf.horizontalAdvance(QStringLiteral("%1 : %2").arg(m_name, m_module)) + 24;
    int needed = left_label_w + right_label_w + kPinShapeSize * 2 + kInstanceCenterPadding + kPinLabelHPadding * 2;
    needed = std::max(needed, header_w);
    // Cap below the column pitch — a wider body invades the wire gutters and
    // over-constrains lane allocation. Labels elide to per-side budgets.
    m_width = std::min(std::max(needed, kMinInstanceWidth), kMaxInstanceWidth);

    int avail = m_width - kPinShapeSize * 2 - kInstanceCenterPadding - kPinLabelHPadding * 2;
    if (left_label_w + right_label_w <= avail) {
        m_left_label_budget = avail - right_label_w;
        m_right_label_budget = avail - left_label_w;
    } else {
        m_left_label_budget = (left_label_w * avail) / (left_label_w + right_label_w);
        m_right_label_budget = avail - m_left_label_budget;
    }

    setRect(0, 0, m_width, kInstanceHeaderHeight + body);
}

void InstanceItem::paint(QPainter *painter, const QStyleOptionGraphicsItem *option, QWidget *) {
    QRectF r = rect();
    painter->setRenderHint(QPainter::Antialiasing);
    bool dirty = m_state->instance_is_dirty_name(m_name);
    bool selected =
        (m_state->selected_instance() == m_name) || (option && (option->state & QStyle::State_Selected));

    painter->setPen(Qt::NoPen);
    painter->setBrush(QColor(36, 39, 44));
    painter->drawRoundedRect(r, 8, 8);

    painter->save();
    QPainterPath body_path;
    body_path.addRoundedRect(r, 8, 8);
    painter->setClipPath(body_path);
    QRectF header_rect(r.left(), r.top(), r.width(), kInstanceHeaderHeight);
    painter->setBrush(QColor(46, 50, 56));
    painter->drawRect(header_rect);
    painter->setPen(QColor(28, 30, 34));
    painter->drawLine(QPointF(r.left(), r.top() + kInstanceHeaderHeight),
                      QPointF(r.right(), r.top() + kInstanceHeaderHeight));
    painter->restore();

    QPen pen;
    if (dirty) {
        pen.setColor(QColor(220, 90, 80));
        pen.setWidthF(selected ? 1.8 : 1.2);
    } else if (selected) {
        pen.setColor(QColor(120, 170, 240));
        pen.setWidthF(1.8);
    } else {
        pen.setColor(QColor(60, 64, 70));
        pen.setWidthF(1.0);
    }
    pen.setCosmetic(true);
    painter->setPen(pen);
    painter->setBrush(Qt::NoBrush);
    painter->drawRoundedRect(r, 8, 8);

    QFont name_font = painter->font();
    name_font.setBold(true);
    name_font.setPointSizeF(name_font.pointSizeF() + 0.5);
    painter->setFont(name_font);
    painter->setPen(QColor(232, 234, 238));
    painter->drawText(QRectF(r.left() + 12, r.top() + 8, r.width() - 24, 20),
                      Qt::AlignLeft | Qt::AlignVCenter,
                      painter->fontMetrics().elidedText(m_name, Qt::ElideRight, static_cast<int>(r.width()) - 24));

    QFont mod_font = painter->font();
    mod_font.setBold(false);
    mod_font.setPointSizeF(mod_font.pointSizeF() - 1.5);
    painter->setFont(mod_font);
    painter->setPen(QColor(150, 154, 162));
    painter->drawText(QRectF(r.left() + 12, r.top() + 30, r.width() - 24, 16),
                      Qt::AlignLeft | Qt::AlignVCenter,
                      painter->fontMetrics().elidedText(m_module, Qt::ElideRight, static_cast<int>(r.width()) - 24));

    // Collapsed group blocks: accent border + '+' button to expand back
    // into the hull view. (Expanded groups draw their '−' on the hull.)
    if (m_state->is_group_block(m_name)) {
        QPen accent(QColor(170, 140, 220), selected ? 2.0 : 1.4);
        accent.setCosmetic(true);
        painter->setPen(accent);
        painter->setBrush(Qt::NoBrush);
        painter->drawRoundedRect(r.adjusted(2, 2, -2, -2), 6, 6);

        QRectF g = toggleGlyphRect();
        painter->setPen(QPen(QColor(150, 154, 162), 1.0));
        painter->setBrush(QColor(52, 56, 62));
        painter->drawRoundedRect(g, 4, 4);
        painter->setPen(QColor(226, 228, 232));
        QFont gf = painter->font();
        gf.setBold(true);
        painter->setFont(gf);
        painter->drawText(g, Qt::AlignCenter, QStringLiteral("+"));
    }
}

// WireTool, CanvasView, CanvasLayer all live in canvas.h / canvas.cpp.

PortPinItem::~PortPinItem() {
    if (m_wire_tool) {
        m_wire_tool->notifyPinDestroyed(this);
    }
}

void PortPinItem::flashRed(int ms) {
    m_flash = true;
    update();
    QTimer::singleShot(ms, [this]() {
        m_flash = false;
        update();
    });
}

void PortPinItem::mousePressEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_wire_tool) {
        m_wire_tool->onPinPressed(this, event->scenePos());
        event->accept();
        return;
    }
    QGraphicsItem::mousePressEvent(event);
}

void PortPinItem::mouseMoveEvent(QGraphicsSceneMouseEvent *event) {
    if ((event->buttons() & Qt::LeftButton) && m_wire_tool) {
        m_wire_tool->onPinDragMove(event->scenePos());
        event->accept();
        return;
    }
    QGraphicsItem::mouseMoveEvent(event);
}

void PortPinItem::mouseReleaseEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_wire_tool) {
        m_wire_tool->onPinReleased(event->scenePos());
        event->accept();
        return;
    }
    QGraphicsItem::mouseReleaseEvent(event);
}

// Prompt the user to create or edit a manual bundle on `instance`. If
// `preselected_port` is non-empty it starts checked. Returns true on success.
static bool prompt_create_manual_bundle(QWidget *parent, AppState *state, const QString &instance,
                                        const QString &preselected_port) {
    int idx = find_instance_index(state, instance);
    if (idx < 0)
        return false;
    int n = state->instance_port_count(idx);
    if (n == 0)
        return false;

    QDialog dlg(parent);
    dlg.setWindowTitle(QStringLiteral("Group into interface"));

    auto *name_edit = new QLineEdit(&dlg);
    name_edit->setPlaceholderText(QStringLiteral("e.g. spi, m_axi, s_axi_lite"));

    auto *box_layout = new QVBoxLayout;
    QList<QCheckBox *> checks;
    checks.reserve(n);
    for (int p = 0; p < n; ++p) {
        QString pname = state->instance_port_name(idx, p);
        auto *cb = new QCheckBox(pname, &dlg);
        if (pname == preselected_port)
            cb->setChecked(true);
        box_layout->addWidget(cb);
        checks.append(cb);
    }

    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dlg);
    QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);

    auto *form = new QFormLayout;
    form->addRow(QStringLiteral("&Bundle name:"), name_edit);

    auto *scroll_inner = new QWidget;
    scroll_inner->setLayout(box_layout);
    auto *scroll = new QScrollArea;
    scroll->setWidget(scroll_inner);
    scroll->setWidgetResizable(true);
    scroll->setMinimumHeight(200);

    auto *layout = new QVBoxLayout(&dlg);
    layout->addLayout(form);
    layout->addWidget(new QLabel(QStringLiteral("Ports:"), &dlg));
    layout->addWidget(scroll);
    layout->addWidget(buttons);

    if (dlg.exec() != QDialog::Accepted)
        return false;
    QString name = name_edit->text().trimmed();
    if (name.isEmpty())
        return false;
    QStringList picked;
    for (auto *cb : checks) {
        if (cb->isChecked())
            picked << cb->text();
    }
    if (picked.size() < 2)
        return false;
    return state->create_manual_bundle(instance, name, picked.join(QChar(',')));
}

// Dialog: enter a driver expression + slice spec for a multi-bit port.
// Returns true if a slice was applied.
static bool prompt_connect_slice(QWidget *parent, AppState *state, const QString &instance, const QString &port) {
    QDialog dlg(parent);
    dlg.setWindowTitle(QStringLiteral("Connect slice"));
    auto *driver_edit = new QLineEdit(&dlg);
    driver_edit->setPlaceholderText(QStringLiteral("driver (e.g. u_counter.count, or clk for a top-port)"));
    auto *slice_edit = new QLineEdit(&dlg);
    slice_edit->setPlaceholderText(QStringLiteral("slice (e.g. 0 or 7:4)"));
    auto *form = new QFormLayout;
    form->addRow(QStringLiteral("Driver:"), driver_edit);
    form->addRow(QStringLiteral("Slice:"), slice_edit);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dlg);
    QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);
    auto *layout = new QVBoxLayout(&dlg);
    layout->addLayout(form);
    layout->addWidget(buttons);
    if (dlg.exec() != QDialog::Accepted)
        return false;

    QString driver = driver_edit->text().trimmed();
    QString slice = slice_edit->text().trimmed();
    slice.remove(QChar('[')).remove(QChar(']'));
    if (driver.isEmpty() || slice.isEmpty())
        return false;

    // driver_inst / driver_port split on first '.'
    QString driver_inst, driver_port;
    int dot = driver.indexOf(QChar('.'));
    if (dot < 0) {
        driver_inst = QString(); // top-port
        driver_port = driver;
    } else {
        driver_inst = driver.left(dot);
        driver_port = driver.mid(dot + 1);
    }

    // Parse slice: "h:l" or single integer.
    int high = 0, low = 0;
    int colon = slice.indexOf(QChar(':'));
    bool ok = false;
    if (colon < 0) {
        int v = slice.toInt(&ok);
        if (!ok)
            return false;
        high = v;
        low = v;
    } else {
        high = slice.left(colon).trimmed().toInt(&ok);
        if (!ok)
            return false;
        low = slice.mid(colon + 1).trimmed().toInt(&ok);
        if (!ok)
            return false;
    }
    return state->set_port_map_entry_slice(instance, port, driver_inst, driver_port, high, low);
}

void PortPinItem::contextMenuEvent(QGraphicsSceneContextMenuEvent *event) {
    if (!m_parent || !m_parent->state()) {
        return;
    }
    AppState *state = m_parent->state();
    QString inst = m_parent->instanceName();
    bool is_bundle = dynamic_cast<BundlePinItem *>(this) != nullptr;

    QMenu menu;
    QAction *ungroupAct = nullptr;
    QAction *promoteAct = nullptr;
    QAction *clearAct = nullptr;
    QAction *sliceAct = nullptr;
    QAction *constAct = nullptr;
    if (is_bundle) {
        ungroupAct = menu.addAction(QStringLiteral("Ungroup"));
    } else {
        // Grouping lives on the module's context menu (InstanceItem) — it needs
        // a fresh multi-port selection, not a single pre-picked pin.
        promoteAct = menu.addAction(QStringLiteral("Promote to top-level port"));
        // Slice dialog makes sense only for multi-bit ports (width != 0).
        if (m_width != 0) {
            sliceAct = menu.addAction(QStringLiteral("Connect slice..."));
        }
        if (m_direction == 0 /*In*/) {
            constAct = menu.addAction(QStringLiteral("Tie to constant..."));
        }
        clearAct = menu.addAction(QStringLiteral("Clear connection"));
    }
    QAction *chosen = menu.exec(event->screenPos());
    event->accept();
    if (!chosen)
        return;

    QWidget *parent_w = nullptr;
    if (auto *s = scene()) {
        if (!s->views().isEmpty())
            parent_w = s->views().first()->window();
    }
    const QString port = m_name;

    // These mutate AppState, which synchronously relayouts pins and deletes
    // THIS item — doing that inside the pin's own event handler is a
    // use-after-free. Defer to after the handler returns. Capture by value
    // (never `this`).
    if (chosen == promoteAct) {
        QTimer::singleShot(0, [state, inst, port]() {
            QString resolved = state->promote_port_to_top(inst, port);
            if (!resolved.isEmpty() && resolved != port)
                QToolTip::showText(QCursor::pos(), QStringLiteral("Promoted as '%1'").arg(resolved));
        });
    } else if (chosen == ungroupAct) {
        QTimer::singleShot(0, [state, inst, port]() { state->remove_manual_bundle(inst, port); });
    } else if (chosen == clearAct) {
        QTimer::singleShot(0, [state, inst, port]() { state->clear_port_map_entry(inst, port); });
    } else if (chosen == sliceAct) {
        QTimer::singleShot(0, [state, inst, port, parent_w]() {
            prompt_connect_slice(parent_w, state, inst, port);
        });
    } else if (chosen == constAct) {
        const int width = m_width;
        QTimer::singleShot(0, [state, inst, port, parent_w, width]() {
            // Language-appropriate default: scalar '0' / 1'b0, vector zeros.
            bool sv = state->project_language() == 1;
            QString def;
            if (width == 0) {
                def = sv ? QStringLiteral("1'b0") : QStringLiteral("'0'");
            } else if (sv) {
                def = QStringLiteral("%1'b0").arg(width > 0 ? width : 1);
            } else {
                def = QStringLiteral("(others => '0')");
            }
            bool ok = false;
            QString lit = QInputDialog::getText(
                parent_w, QStringLiteral("Tie to constant"),
                QStringLiteral("Literal (emitted verbatim, e.g. %1):").arg(def),
                QLineEdit::Normal, def, &ok);
            lit = lit.trimmed();
            if (ok && !lit.isEmpty())
                state->set_port_map_entry(inst, port, lit);
        });
    }
}

void InstanceItem::commitDragPosition() {
    // Multi-drag: every selected instance keeps its dropped spot (relative
    // spacing intact); the rest of the column shoves out of the way.
    QList<InstanceItem *> fixed;
    if (auto *sc = scene()) {
        for (QGraphicsItem *it : sc->selectedItems()) {
            if (auto *ii = dynamic_cast<InstanceItem *>(it))
                fixed << ii;
        }
    }
    if (!fixed.contains(this))
        fixed << this;
    if (m_canvas_layer) {
        m_canvas_layer->settleAfterMove(fixed);
    } else {
        QPointF p = pos();
        m_state->set_instance_position(m_name, p.x(), p.y());
    }
}

void InstanceItem::toggleBundleExpanded(const QString &bundle) {
    if (m_expanded_bundles.contains(bundle))
        m_expanded_bundles.remove(bundle);
    else
        m_expanded_bundles.insert(bundle);
    relayoutPins();
    // Anchors moved (members <-> header): reroute so collapsed members merge
    // into one bus wire and expanded members split back out.
    if (m_canvas_layer)
        m_canvas_layer->replanWires();
}

void InstanceItem::contextMenuEvent(QGraphicsSceneContextMenuEvent *event) {
    if (!m_state) {
        event->ignore();
        return;
    }
    bool group_block = m_state->is_group_block(m_name);
    QString member_group = group_block ? QString() : m_state->instance_group(m_name);

    QMenu menu;
    QAction *groupPortsAct = nullptr;
    QAction *expandAct = nullptr;
    QAction *ungroupAct = nullptr;
    QAction *collapseAct = nullptr;
    QAction *createGroupAct = nullptr;
    if (group_block) {
        expandAct = menu.addAction(QStringLiteral("Expand group"));
        ungroupAct = menu.addAction(QStringLiteral("Ungroup"));
    } else {
        groupPortsAct = menu.addAction(QStringLiteral("Group ports into interface..."));
        if (!member_group.isEmpty()) {
            collapseAct =
                menu.addAction(QStringLiteral("Collapse group '%1'").arg(member_group));
        } else {
            createGroupAct = menu.addAction(QStringLiteral("Group into module..."));
        }
    }
    QAction *chosen = menu.exec(event->screenPos());
    event->accept();
    if (!chosen)
        return;

    QWidget *parent_w = nullptr;
    if (auto *s = scene()) {
        if (!s->views().isEmpty())
            parent_w = s->views().first()->window();
    }
    AppState *st = m_state;
    const QString name = m_name;

    // Group mutations rebuild the canvas and delete THIS item — defer past
    // the handler, capture by value, never `this` (except reading selection
    // synchronously below, before any mutation).
    if (chosen == groupPortsAct) {
        prompt_create_manual_bundle(parent_w, m_state, m_name, QString());
    } else if (chosen == expandAct) {
        QTimer::singleShot(0, [st, name]() { st->set_group_collapsed(name, false); });
    } else if (chosen == ungroupAct) {
        QTimer::singleShot(0, [st, name]() { st->remove_group(name); });
    } else if (chosen == collapseAct) {
        const QString grp = member_group;
        QTimer::singleShot(0, [st, grp]() { st->set_group_collapsed(grp, true); });
    } else if (chosen == createGroupAct) {
        // Members = current multi-selection plus this block.
        QStringList members;
        if (auto *sc = scene()) {
            for (QGraphicsItem *it : sc->selectedItems()) {
                if (auto *inst = dynamic_cast<InstanceItem *>(it)) {
                    if (!inst->m_state->is_group_block(inst->instanceName()))
                        members << inst->instanceName();
                }
            }
        }
        if (!members.contains(name))
            members << name;
        bool ok = false;
        QString gname = QInputDialog::getText(
            parent_w, QStringLiteral("Group into module"),
            QStringLiteral("Module name for group of %1 instance(s):").arg(members.size()),
            QLineEdit::Normal, QStringLiteral("grp_%1").arg(name), &ok);
        gname = gname.trimmed();
        if (ok && !gname.isEmpty()) {
            const QString joined = members.join(QStringLiteral("\n"));
            QTimer::singleShot(0, [st, gname, joined]() {
                st->create_group(gname, joined);
            });
        }
    }
}

void TopPortItem::paintIssueRing(QPainter *painter, const QPointF &center) {
    AppState *state = m_layer ? m_layer->state() : nullptr;
    if (!state)
        return;
    int issue = state->pin_issue_level(key());
    QString tip = portName() + format_width(width());
    if (issue > 0) {
        painter->setBrush(Qt::NoBrush);
        QPen ring(issue >= 2 ? QColor(232, 72, 60) : QColor(255, 179, 0), 2.0);
        painter->setPen(ring);
        painter->drawEllipse(center, kPinShapeSize + 3, kPinShapeSize + 3);
        tip += QStringLiteral("\n") + state->pin_issue_message(key());
    }
    if (tip != toolTip())
        setToolTip(tip);
}

// WireTool impls + parse_pin_key moved to canvas.cpp.


// WireItem::contextMenuEvent moved inline to items.h.
// CanvasView + CanvasLayer moved to canvas.h / canvas.cpp.


// --- InstanceItem::itemChange (live wire reroute during drag) ---------------

QVariant InstanceItem::itemChange(GraphicsItemChange change, const QVariant &value) {
    if (change == ItemPositionChange) {
        // Live placement: snap X to the column grid, but let Y follow the cursor.
        // Vertical collision-shoving is deferred to release/drop (see
        // commitDragPosition / placeInstance) so dragging within a rank stays
        // smooth instead of fighting the cursor each frame.
        QPointF p = value.toPointF();
        qreal centered = p.x() + m_width / 2.0;
        int col = static_cast<int>(std::round(centered / kColumnPitch));
        return QPointF(col * kColumnPitch - m_width / 2.0, p.y());
    }
    if (change == ItemPositionHasChanged && m_canvas_layer) {
        m_canvas_layer->onInstanceColumnChanged();
    }
    return QGraphicsRectItem::itemChange(change, value);
}

} // namespace hdlc
