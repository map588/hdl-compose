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
    QString k = pin->key();
    int dot = k.indexOf(QChar('.'));
    if (dot < 0)
        return false;
    *inst = k.left(dot);
    *port = k.mid(dot + 1);
    return true;
}

} // namespace

// --- WireTool ---------------------------------------------------------------

QString WireTool::compatibilityError(PortPinItem *src, PortPinItem *dst) const {
    if (src == dst) {
        return QStringLiteral("cannot connect pin to itself");
    }
    bool src_top = src->key().startsWith(QStringLiteral("top:"));
    bool dst_top = dst->key().startsWith(QStringLiteral("top:"));
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
    bool src_top = src->key().startsWith(QStringLiteral("top:"));
    bool dst_top = dst->key().startsWith(QStringLiteral("top:"));

    if (src_top && dst_top) {
        QToolTip::showText(QCursor::pos(), QStringLiteral("cannot wire two top-level ports"));
        return false;
    }

    if (src_top || dst_top) {
        PortPinItem *top = src_top ? src : dst;
        PortPinItem *inst_pin = src_top ? dst : src;
        QString inst_key = inst_pin->key();
        int dot = inst_key.indexOf(QChar('.'));
        if (dot < 0)
            return false;
        QString inst = inst_key.left(dot);
        QString port = inst_key.mid(dot + 1);
        QString top_name = top->key().mid(4);
        m_state->set_port_map_entry(inst, port, top_name);
        return true;
    }

    if (src->direction() == 0 && dst->direction() == 0) {
        return tryCommitMultiLoad(src, dst);
    }
    auto can_drive = [](int dir) { return dir == 1 || dir == 2; };
    PortPinItem *driver = can_drive(src->direction()) ? src : dst;
    PortPinItem *load = (driver == src) ? dst : src;
    QString load_key = load->key();
    int dot = load_key.indexOf(QChar('.'));
    if (dot < 0)
        return false;
    QString inst = load_key.left(dot);
    QString port = load_key.mid(dot + 1);
    QString dkey = driver->key();
    QString driver_rhs = dkey.startsWith(QStringLiteral("top:")) ? dkey.mid(4) : dkey;
    if (driver->width() == 1 && load->width() == 0) {
        int dd = dkey.indexOf(QChar('.'));
        if (dd > 0 && !dkey.startsWith(QStringLiteral("top:"))) {
            QString d_inst = dkey.left(dd);
            QString d_port = dkey.mid(dd + 1);
            m_state->set_port_map_entry_slice(inst, port, d_inst, d_port, 0, 0);
            return true;
        }
    }
    m_state->set_port_map_entry(inst, port, driver_rhs);
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
    QString rhs = QStringLiteral("%1.%2").arg(a_inst, a_port);
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
            if (pin != m_armed) {
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
        clearProvisional();
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
    qreal centered = scene_pos.x();
    int col = static_cast<int>(std::round(centered / kColumnPitch));
    qreal snapped_x = col * kColumnPitch - kMinInstanceWidth / 2.0;
    m_state->set_instance_position(inst_name, snapped_x, scene_pos.y());
    event->acceptProposedAction();
}

} // namespace hdlc
