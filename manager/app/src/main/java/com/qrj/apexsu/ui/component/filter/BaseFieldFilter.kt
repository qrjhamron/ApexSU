package com.qrj.apexsu.ui.component.filter

import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue

open class BaseFieldFilter() {
    private var inputValue = mutableStateOf(TextFieldValue())

    constructor(value: String) : this() {
        inputValue.value = TextFieldValue(value, TextRange(value.lastIndex + 1))
    }

    protected open fun onFilter(inputTextFieldValue: TextFieldValue, lastTextFieldValue: TextFieldValue): TextFieldValue {
        return TextFieldValue()
    }

    protected open fun computePos(
        lastText: String,
        inputTextFieldValue: TextFieldValue,
        newText: String
    ): Int {
        val selection = inputTextFieldValue.selection
        if (!selection.collapsed) {
            return newText.length
        }

        val cursor = selection.end
        val diff = newText.length - lastText.length
        
        // Basic heuristic: if text grew, move cursor forward; if shrunk, try to maintain relative position
        var newPos = cursor + diff
        if (newPos < 0) newPos = 0
        if (newPos > newText.length) newPos = newText.length
        
        return newPos
    }

    protected fun getNewTextRange(
        lastTextFieldValue: TextFieldValue,
        inputTextFieldValue: TextFieldValue
    ): TextRange? {
        val lastText = lastTextFieldValue.text
        val inputText = inputTextFieldValue.text
        
        if (lastText == inputText) return null
        
        // Find the start of the change
        var start = 0
        while (start < lastText.length && start < inputText.length && lastText[start] == inputText[start]) {
            start++
        }
        
        return TextRange(start, inputText.length)
    }

    protected fun getNewText(
        lastTextFiled: TextFieldValue,
        inputTextFieldValue: TextFieldValue
    ): TextRange? {

        return null
    }

    fun setInputValue(value: String) {
        inputValue.value = TextFieldValue(value, TextRange(value.lastIndex + 1))
    }

    fun getInputValue(): TextFieldValue {
        return inputValue.value
    }

    fun onValueChange(): (TextFieldValue) -> Unit {
        return {
            inputValue.value = onFilter(it, inputValue.value)
        }
    }
}
