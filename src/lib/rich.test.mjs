import assert from 'node:assert/strict';
import { parseRich, plainRich } from './rich.js';

const T = (text) => ({ kind: 'text', text });
const B = (text) => ({ kind: 'b', text });
const N = (text) => ({ kind: 'neon', text });

// обычный текст
assert.deepEqual(parseRich('привет мир'), [T('привет мир')]);
// жирный
assert.deepEqual(parseRich('**a**'), [B('a')]);
// неон
assert.deepEqual(parseRich('==b=='), [N('b')]);
// смесь
assert.deepEqual(parseRich('x **a** y ==b== z'), [T('x '), B('a'), T(' y '), N('b'), T(' z')]);
// незакрытый маркер -> хвост текстом
assert.deepEqual(parseRich('**a'), [T('**a')]);
assert.deepEqual(parseRich('x ==b'), [T('x ==b')]);
assert.deepEqual(parseRich('**a** ==c'), [B('a'), T(' ==c')]);
// пустые строки и не строки
assert.deepEqual(parseRich(''), []);
assert.deepEqual(parseRich(null), []);
assert.deepEqual(parseRich(undefined), []);
// пустые токены не выдаются
assert.deepEqual(parseRich('****'), []);
assert.deepEqual(parseRich('a****b'), [T('ab')]);
// реальный кейс
assert.deepEqual(parseRich('==+5 CX== за голос'), [N('+5 CX'), T(' за голос')]);

// plainRich
assert.equal(plainRich('x **a** y ==b== z'), 'x a y b z');
assert.equal(plainRich('**a'), '**a');
assert.equal(plainRich('==+5 CX== за голос'), '+5 CX за голос');
assert.equal(plainRich(''), '');
assert.equal(plainRich(null), '');
assert.equal(plainRich(undefined), '');

console.log('OK');
