import React, { useEffect, useState } from 'react';
import { Box, Text, useInput } from 'ink';
import type { Key } from 'ink';
import type { ColorMode } from '../types.js';
import { palette, tone } from './theme.js';

export type ComposerEdit = { value: string; cursor: number; submit?: boolean };

export function editComposer(value: string, cursor: number, input: string, key: Pick<Key, 'backspace' | 'delete' | 'leftArrow' | 'rightArrow' | 'return' | 'ctrl' | 'shift'>): ComposerEdit {
  if (key.return && (key.ctrl || !key.shift)) return { value, cursor, submit: true };
  if (key.return && key.shift) return { value: `${value.slice(0, cursor)}\n${value.slice(cursor)}`, cursor: cursor + 1 };
  if (key.leftArrow) return { value, cursor: Math.max(0, cursor - 1) };
  if (key.rightArrow) return { value, cursor: Math.min(value.length, cursor + 1) };
  if (key.ctrl && input === 'a') return { value, cursor: value.lastIndexOf('\n', cursor - 1) + 1 };
  if (key.ctrl && input === 'e') { const end = value.indexOf('\n', cursor); return { value, cursor: end === -1 ? value.length : end }; }
  if (key.ctrl && input === 'u') return { value: '', cursor: 0 };
  if (key.ctrl && input === 'w') { const start = value.slice(0, cursor).replace(/\s+$/, '').search(/\S+$/); const from = start < 0 ? 0 : start; return { value: value.slice(0, from) + value.slice(cursor), cursor: from }; }
  if (key.backspace && cursor > 0) return { value: value.slice(0, cursor - 1) + value.slice(cursor), cursor: cursor - 1 };
  if (key.delete && cursor < value.length) return { value: value.slice(0, cursor) + value.slice(cursor + 1), cursor };
  if (input && !key.ctrl) return { value: value.slice(0, cursor) + input + value.slice(cursor), cursor: cursor + input.length };
  return { value, cursor };
}

export function Composer({ value, onChange, onSubmit, width, colorMode, focused, disabled, placeholder }: { value: string; onChange: (value: string) => void; onSubmit: (value: string) => void; width: number; colorMode: ColorMode; focused: boolean; disabled: boolean; placeholder: string }) {
  const [cursor, setCursor] = useState(value.length);
  useEffect(() => setCursor(current => Math.min(current, value.length)), [value]);
  useInput((input, key) => {
    if (!focused || disabled || key.escape || key.tab || key.upArrow || key.downArrow || key.pageUp || key.pageDown) return;
    const next = editComposer(value, cursor, input, key);
    if (next.submit) { onSubmit(value); setCursor(0); return; }
    if (next.value !== value) onChange(next.value);
    setCursor(next.cursor);
  }, { isActive: focused && !disabled });

  const before = value.slice(0, cursor);
  const current = value[cursor] ?? ' ';
  const after = value.slice(cursor + (cursor < value.length ? 1 : 0));
  return (
    <Box borderStyle="round" borderColor={tone(focused ? palette.borderFocus : palette.border, colorMode)} paddingX={1} width={Math.max(20, width)} minHeight={3}>
      <Text color={tone(palette.accent, colorMode)}>{'>'} </Text>
      {value.length === 0 && !focused ? <Text color={tone(palette.muted, colorMode)}>{placeholder}</Text> :
        value.length === 0 ? <Text><Text inverse> </Text><Text color={tone(palette.muted, colorMode)}>{placeholder}</Text></Text> :
          <Text color={tone(palette.text, colorMode)}>{before}<Text inverse={focused}>{current}</Text>{after}</Text>}
    </Box>
  );
}
