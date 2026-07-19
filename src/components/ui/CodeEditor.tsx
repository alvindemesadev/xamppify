import { useEffect, useRef } from "react";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { oneDark } from "@codemirror/theme-one-dark";
import { sql } from "@codemirror/lang-sql";
import { html } from "@codemirror/lang-html";
import { javascript } from "@codemirror/lang-javascript";
import { php } from "@codemirror/lang-php";
import { xml } from "@codemirror/lang-xml";
import { css } from "@codemirror/lang-css";
import { json } from "@codemirror/lang-json";
import { autocompletion, closeBrackets } from "@codemirror/autocomplete";

type Language = "sql" | "html" | "js" | "php" | "xml" | "css" | "json" | "text";

const langs: Record<string, () => import("@codemirror/language").LanguageSupport> = {
  sql, html, js: javascript, php, xml, css, json,
};

function detectLanguage(filename: string): Language {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  if (["sql"].includes(ext)) return "sql";
  if (["html", "htm", "shtml"].includes(ext)) return "html";
  if (["js", "mjs", "cjs", "jsx"].includes(ext)) return "js";
  if (["php", "phtml", "php3", "php4", "php5", "phps"].includes(ext)) return "php";
  if (["xml", "svg", "xsd", "xsl", "xslt"].includes(ext)) return "xml";
  if (["css", "scss", "less", "sass"].includes(ext)) return "css";
  if (["json"].includes(ext)) return "json";
  if (["conf", "ini", "cfg", "txt", "md", "log"].includes(ext)) return "text";
  return "text";
}

interface CodeEditorProps {
  value: string;
  onChange?: (value: string) => void;
  filename?: string;
  language?: Language;
  readOnly?: boolean;
  placeholder?: string;
}

export function CodeEditor({
  value,
  onChange,
  filename,
  language,
  readOnly = false,
  placeholder: placeholderText,
}: CodeEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  const lang = language ?? (filename ? detectLanguage(filename) : "text");

  useEffect(() => {
    if (!editorRef.current) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        onChange?.(update.state.doc.toString());
      }
    });

    const extensions = [
      keymap.of([...defaultKeymap, ...historyKeymap]),
      history(),
      closeBrackets(),
      autocompletion(),
      EditorView.lineWrapping,
      oneDark,
      updateListener,
    ];

    if (placeholderText) {
      extensions.push(placeholder(placeholderText));
    }

    if (readOnly) {
      extensions.push(EditorView.editable.of(false));
    }

    const langSupport = langs[lang];
    if (langSupport) {
      extensions.push(langSupport());
    }

    const state = EditorState.create({
      doc: value,
      extensions,
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // Only re-create when language/readOnly/placeholder changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lang, readOnly, placeholderText]);

  useEffect(() => {
    const view = viewRef.current;
    if (view && value !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: value },
      });
    }
  }, [value]);

  return <div ref={editorRef} className="h-full overflow-auto" />;
}
