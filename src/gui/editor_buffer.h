// Mini-editor buffer subsystem: rendering an instance's generic/port map
// as an editable text buffer, parsing it back, inline error highlighting,
// and completion context detection. Pure functions + one QSyntaxHighlighter;
// no widget code (that lives in MainWindow in app.cpp).

#pragma once

#include <QList>
#include <QPair>
#include <QSet>
#include <QString>
#include <QStringList>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>

#include "hdl-compose/src/gui/bridge.cxxqt.h"

namespace hdlc {

// Result of parsing an editor buffer. Errors carry the offending line
// number so the inline highlighter can underline the right block.
struct EditorParseResult {
    QList<QPair<QString, QString>> generic_commits; // (name, rhs)
    QList<QPair<QString, QString>> port_commits;    // (name, rhs_clean)
    QList<QPair<int, QString>> errors;              // (line_index_0based, message)
};

// Detect completer context at the cursor. Returns kind + prefix to filter by.
//   None    — cursor not in a completable spot; popup should hide.
//   Rhs     — anywhere in RHS of `=>` line; offer all drivers.
//   DotPort — right after `<inst>.`; offer that instance's ports only.
struct CompletionContext {
    enum Kind { None, Rhs, DotPort } kind = None;
    QString prefix;   // chars typed so far (popup filter)
    QString instance; // for DotPort: the instance name before the dot
};

// Underlines the RHS of port-map lines that fail validation. The set of
// bad line indices is recomputed on every textChanged in the editor.
class MiniEditorHighlighter : public QSyntaxHighlighter {
  public:
    explicit MiniEditorHighlighter(QTextDocument *doc) : QSyntaxHighlighter(doc) {}

    void setErrorLines(const QSet<int> &lines) {
        if (lines == m_error_lines)
            return;
        m_error_lines = lines;
        rehighlight();
    }

  protected:
    void highlightBlock(const QString &text) override {
        int blk = currentBlock().blockNumber();
        if (!m_error_lines.contains(blk))
            return;
        QTextCharFormat fmt;
        fmt.setUnderlineColor(QColor(220, 60, 60));
        fmt.setUnderlineStyle(QTextCharFormat::WaveUnderline);
        int start = 0;
        int len = text.length();
        int arrow = text.indexOf(QStringLiteral("=>"));
        if (arrow >= 0) {
            // VHDL form: underline RHS after `=>`.
            start = arrow + 2;
            while (start < text.length() && text[start].isSpace())
                ++start;
            len = text.length() - start;
        } else {
            // SV form `.name(value)`: underline the value between parens.
            int open = text.indexOf(QChar('('));
            int close = text.lastIndexOf(QChar(')'));
            if (open >= 0 && close > open) {
                start = open + 1;
                len = close - start;
            }
        }
        if (len > 0)
            setFormat(start, len, fmt);
    }

  private:
    QSet<int> m_error_lines;
};

// Render one instance's bindings as a VHDL/SV component-instantiation buffer.
// Empty string if `instance_name` is empty or unknown.
QString build_instance_buffer(AppState *state, const QString &instance_name);

EditorParseResult parse_editor_buffer(const QString &buffer);

// Commit a buffer back to the model. Returns error messages; empty on
// success. On any parse error the model is NOT mutated.
QStringList commit_editor_buffer(AppState *state, const QString &instance_name, const QString &buffer);

CompletionContext detect_completion_context(const QString &line_before_cursor);

// All RHS driver candidates: top-port names + "<instance>.<port>" strings,
// excluding the instance being edited.
QStringList rhs_candidates(AppState *state, const QString &editing_inst);

// Ports of one instance, for dot-completion after "<inst>.".
QStringList instance_port_candidates(AppState *state, const QString &inst_name);

} // namespace hdlc
