// Chip view: the device as nextpnr placed the design on it.
//
// The data is `<top>.floorplan.json`, which `hdl-compose build` writes (see
// `src/floorplan.rs`). Free bels are grey outlines. Each tile that holds
// cells of the design gets a colored square; the color is the cell kind
// that the tile holds most of. The used bels inside the tile are solid.
#pragma once

#include <QFile>
#include <QGraphicsItem>
#include <QGraphicsRectItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNativeGestureEvent>
#include <QPainter>
#include <QSet>
#include <QStringList>
#include <QStyleOptionGraphicsItem>
#include <QVector>
#include <QWheelEvent>
#include <cmath>

namespace chipview {

    // Order of `CellKind` in `src/floorplan.rs`.
    inline const QStringList& kindNames() {
        static const QStringList names {QStringLiteral("Logic"), QStringLiteral("Register"),
            QStringLiteral("Io"), QStringLiteral("Dsp"), QStringLiteral("Memory"),
            QStringLiteral("Clock"), QStringLiteral("Other")};
        return names;
    }

    inline QColor kindColor(int kind) {
        static const QColor colors[] = {QColor(0x3b, 0x82, 0xf6), QColor(0x06, 0xb6, 0xd4),
            QColor(0x22, 0xc5, 0x5e), QColor(0xa8, 0x55, 0xf7), QColor(0xf5, 0x9e, 0x0b),
            QColor(0xef, 0x44, 0x44), QColor(0x9c, 0xa3, 0xaf)};
        return colors[kind >= 0 && kind < 7 ? kind : 6];
    }

    inline const QColor kBackground(24, 24, 28);
    inline const QColor kFreeBel(92, 92, 100);
    inline const QColor kHighlight(250, 204, 21);

    // All bel outlines in one item. The device has about 100 000 bels, so the
    // item skips each bel that is smaller than two pixels at the current zoom.
    class BelLayer : public QGraphicsItem {
    public:
        BelLayer(QVector<QRectF> free, QVector<QRectF> used, QRectF bounds)
            : m_free(std::move(free))
            , m_used(std::move(used))
            , m_bounds(bounds) { }

        QRectF boundingRect() const override { return m_bounds; }

        void paint(QPainter* painter, const QStyleOptionGraphicsItem* option, QWidget*) override {
            const qreal lod
                = QStyleOptionGraphicsItem::levelOfDetailFromTransform(painter->worldTransform());
            const QRectF exposed = option->exposedRect;
            const auto visible = [&](const QRectF& r) {
                return std::max(r.width(), r.height()) * lod >= 2.0 && r.intersects(exposed);
            };
            painter->setPen(QPen(kFreeBel, 0));
            painter->setBrush(Qt::NoBrush);
            for (const QRectF& r : m_free) {
                if (visible(r))
                    painter->drawRect(r);
            }
            painter->setPen(QPen(Qt::white, 0));
            painter->setBrush(QColor(0x60, 0xa5, 0xfa));
            for (const QRectF& r : m_used) {
                if (visible(r))
                    painter->drawRect(r);
            }
        }

    private:
        QVector<QRectF> m_free;
        QVector<QRectF> m_used;
        QRectF m_bounds;
    };

    class FloorplanView : public QGraphicsView {
    public:
        explicit FloorplanView(QWidget* parent = nullptr)
            : QGraphicsView(parent) {
            setScene(new QGraphicsScene(this));
            setBackgroundBrush(kBackground);
            setRenderHint(QPainter::Antialiasing, false);
            setDragMode(QGraphicsView::ScrollHandDrag);
            setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
            setViewportUpdateMode(QGraphicsView::SmartViewportUpdate);
        }

        // Load a chip view file. Returns false and leaves the view empty when
        // the file is absent or is not a chip view.
        bool load(const QString& path) {
            scene()->clear();
            m_tiles.clear();
            m_summary.clear();
            QFile f(path);
            if (!f.open(QIODevice::ReadOnly))
                return false;
            const QJsonObject root = QJsonDocument::fromJson(f.readAll()).object();
            const qreal pitch = root.value(QStringLiteral("tile_pitch")).toDouble();
            if (pitch <= 0)
                return false;

            const auto rects = [&](const char* key) {
                const QJsonArray a = root.value(QLatin1String(key)).toArray();
                QVector<QRectF> out;
                out.reserve(a.size() / 4);
                for (int i = 0; i + 3 < a.size(); i += 4)
                    out.append(QRectF(a[i].toDouble(), a[i + 1].toDouble(), a[i + 2].toDouble(),
                        a[i + 3].toDouble()));
                return out;
            };
            const QRectF bounds(0, 0, root.value(QStringLiteral("width")).toDouble(),
                root.value(QStringLiteral("height")).toDouble());
            scene()->setSceneRect(bounds.adjusted(-pitch, -pitch, pitch, pitch));
            scene()->addItem(new BelLayer(rects("free"), rects("used"), bounds));

            struct Tile {
                int kinds[7] = {};
                QStringList lines;
                QSet<QString> owners;
            };
            QHash<QPair<int, int>, Tile> tiles;
            const QJsonArray cells = root.value(QStringLiteral("cells")).toArray();
            for (const QJsonValue& v : cells) {
                const QJsonObject c = v.toObject();
                Tile& t = tiles[{
                    c.value(QStringLiteral("x")).toInt(), c.value(QStringLiteral("y")).toInt()}];
                const int kind = kindNames().indexOf(c.value(QStringLiteral("kind")).toString());
                t.kinds[kind < 0 ? 6 : kind]++;
                t.lines << QStringLiteral("%1  %2  %3")
                               .arg(c.value(QStringLiteral("bel")).toString(),
                                   c.value(QStringLiteral("cell_type")).toString(),
                                   c.value(QStringLiteral("name")).toString());
                for (const QJsonValue& seg : c.value(QStringLiteral("path")).toArray())
                    t.owners.insert(seg.toString());
            }
            for (auto it = tiles.cbegin(); it != tiles.cend(); ++it) {
                const Tile& t = it.value();
                const int kind = int(std::max_element(t.kinds, t.kinds + 7) - t.kinds);
                auto* item = scene()->addRect(
                    QRectF(it.key().first * pitch, it.key().second * pitch, pitch, pitch));
                item->setZValue(-1); // Below the bel outlines.
                item->setData(kKindRole, kind);
                QStringList lines = t.lines.mid(0, kTooltipLines);
                if (t.lines.size() > kTooltipLines)
                    lines << QStringLiteral("... %1 more").arg(t.lines.size() - kTooltipLines);
                item->setToolTip(QStringLiteral("Tile X%1/Y%2: %3 cells\n")
                                     .arg(it.key().first)
                                     .arg(it.key().second)
                                     .arg(t.lines.size())
                    + lines.join(QChar('\n')));
                m_tiles.append({item, t.owners});
            }

            QStringList parts;
            const QJsonObject util = root.value(QStringLiteral("utilization")).toObject();
            for (auto it = util.constBegin(); it != util.constEnd(); ++it) {
                const QJsonObject u = it.value().toObject();
                const qint64 used = u.value(QStringLiteral("used")).toInteger();
                if (used > 0)
                    parts << QStringLiteral("%1 %2/%3")
                                 .arg(it.key())
                                 .arg(used)
                                 .arg(u.value(QStringLiteral("available")).toInteger());
            }
            const QJsonObject fmax = root.value(QStringLiteral("fmax")).toObject();
            for (auto it = fmax.constBegin(); it != fmax.constEnd(); ++it)
                parts << QStringLiteral("%1 %2 MHz")
                             .arg(it.key())
                             .arg(it.value().toDouble(), 0, 'f', 1);
            m_summary = parts.join(QStringLiteral("   "));

            highlight(m_highlighted);
            fit();
            return true;
        }

        // Utilization and clock frequency, for the label above the view.
        QString summary() const { return m_summary; }

        // Mark the tiles that hold cells of one schematic instance and dim the
        // other tiles. An empty name shows each tile at full strength.
        void highlight(const QString& instance) {
            m_highlighted = instance;
            bool any = false;
            for (const TileItem& t : std::as_const(m_tiles))
                any = any || t.owners.contains(instance);
            for (const TileItem& t : std::as_const(m_tiles)) {
                const bool hit = any && t.owners.contains(instance);
                QColor fill = kindColor(t.item->data(kKindRole).toInt());
                fill.setAlpha(!any ? 130 : hit ? 210 : 40);
                t.item->setBrush(fill);
                QPen outline(kHighlight, 2);
                outline.setCosmetic(true); // Two pixels at each zoom.
                t.item->setPen(hit ? outline : QPen(Qt::NoPen));
            }
        }

        void fit() {
            if (scene()->items().isEmpty())
                return;
            fitInView(scene()->sceneRect(), Qt::KeepAspectRatio);
            m_auto_fit = true;
        }

    protected:
        void wheelEvent(QWheelEvent* event) override {
            if (event->modifiers() & Qt::ControlModifier) {
                double delta = event->angleDelta().y();
                if (delta == 0)
                    delta = event->pixelDelta().y() * 4.0;
                zoomBy(std::pow(1.15, delta / 120.0));
                event->accept();
            } else {
                QGraphicsView::wheelEvent(event);
            }
        }

        // Trackpad pinch, the same gesture as on the schematic canvas.
        bool viewportEvent(QEvent* event) override {
            if (event->type() == QEvent::NativeGesture) {
                auto* gesture = static_cast<QNativeGestureEvent*>(event);
                if (gesture->gestureType() == Qt::ZoomNativeGesture) {
                    zoomBy(1.0 + gesture->value());
                    event->accept();
                    return true;
                }
            }
            return QGraphicsView::viewportEvent(event);
        }

        void mouseDoubleClickEvent(QMouseEvent*) override { fit(); }

        // Keep the whole device in view until the user zooms. The dock gets
        // its size after the first load.
        void resizeEvent(QResizeEvent* event) override {
            QGraphicsView::resizeEvent(event);
            if (m_auto_fit)
                fit();
        }

    private:
        static constexpr int kKindRole = 0;
        static constexpr int kTooltipLines = 24;

        struct TileItem {
            QGraphicsRectItem* item;
            QSet<QString> owners;
        };

        void zoomBy(double factor) {
            const double next = transform().m11() * factor;
            // From the whole device in a small dock to one bel across the view.
            if (next > 0.02 && next < 400.0) {
                scale(factor, factor);
                m_auto_fit = false;
            }
        }

        QVector<TileItem> m_tiles;
        QString m_summary;
        QString m_highlighted;
        bool m_auto_fit = true;
    };

} // namespace chipview
