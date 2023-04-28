module Icingcake {
	function deleteNode(node: Node) {
		if (node.parentNode !== null) {
			node.parentNode.removeChild(node);
		}
	}

	function updateRemoveFilterButtons(form: HTMLFormElement) {
		// how many filter rows remain? enable/disable the remove-filter buttons
		const filterRowButtonList: NodeListOf<HTMLInputElement> = form.querySelectorAll("p.filter-row input[type=button].remove-button");
		const enableButton = (filterRowButtonList.length > 1);
		for (let i = 0; i < filterRowButtonList.length; i++) {
			filterRowButtonList[i].disabled = !enableButton;
		}
	}

	function removeFilterRow(form: HTMLFormElement, rowP: HTMLParagraphElement) {
		deleteNode(rowP);
		updateRemoveFilterButtons(form);
		updateFilterField(form);
	}

	function addSelectOption(select: HTMLSelectElement, value: string, key?: string|undefined) {
		const opt = document.createElement("option");
		select.appendChild(opt);
		opt.textContent = value;
		if (key !== undefined) {
			opt.value = key;
		}
	}

	function quoteIcingaFilter(s: string): string {
		const bits = ["\""];
		for (let i = 0; i < s.length; i++) {
			let c = s.charAt(i);
			if (c == "\\") {
				bits.push("\\\\");
			} else if (c == "\"") {
				bits.push("\\\"");
			} else {
				bits.push(c);
			}
		}
		bits.push("\"");
		return bits.join("");
	}

	function updateFilterField(form: HTMLFormElement) {
		const filterField = <HTMLInputElement|null>form.querySelector("input.filter-field");
		if (filterField === null) {
			return;
		}

		const criteria: string[] = [];
		const rows: NodeListOf<HTMLParagraphElement> = form.querySelectorAll("p.filter-row");
		for (let r = 0; r < rows.length; r++) {
			const criterionSelect = <HTMLSelectElement|null>rows[r].querySelector("select.criterion");
			const operatorSelect = <HTMLSelectElement|null>rows[r].querySelector("select.operator");
			const valueInput = <HTMLInputElement|null>rows[r].querySelector("input.value");

			if (criterionSelect === null || operatorSelect === null || valueInput === null) {
				continue;
			}

			const quotedValue = quoteIcingaFilter(valueInput.value);
			const operator = operatorSelect.value;
			const criterion = criterionSelect.value;
			let expression = "";
			if (operator === "eq") {
				expression = `${criterion}==${quotedValue}`;
			} else if (operator === "ne") {
				expression = `${criterion}!=${quotedValue}`;
			} else if (operator == "match") {
				expression = `match(${quotedValue},${criterion})`;
			} else if (operator == "nmatch") {
				expression = `!match(${quotedValue},${criterion})`;
			} else {
				continue;
			}

			criteria.push(expression);
		}

		filterField.value = criteria.join(" && ");
	}

	function addFilterRow(form: HTMLFormElement) {
		const filterRowP = document.createElement("p");
		form.appendChild(filterRowP);
		filterRowP.classList.add("filter-row");

		const criterionSelect = document.createElement("select");
		filterRowP.appendChild(criterionSelect);
		criterionSelect.classList.add("criterion");
		addSelectOption(criterionSelect, "Host-Name", "host.name");
		addSelectOption(criterionSelect, "Service-Name", "service.name");
		criterionSelect.addEventListener("change", () => updateFilterField(form));
		criterionSelect.addEventListener("input", () => updateFilterField(form));

		const operatorSelect = document.createElement("select");
		filterRowP.appendChild(operatorSelect);
		operatorSelect.classList.add("operator");
		addSelectOption(operatorSelect, "entspricht", "match");
		addSelectOption(operatorSelect, "entspricht nicht", "nmatch");
		addSelectOption(operatorSelect, "=", "eq");
		addSelectOption(operatorSelect, "\u2260", "ne");
		operatorSelect.addEventListener("change", () => updateFilterField(form));
		operatorSelect.addEventListener("input", () => updateFilterField(form));

		const valueInput = document.createElement("input");
		filterRowP.appendChild(valueInput);
		valueInput.type = "text";
		valueInput.classList.add("value");
		valueInput.addEventListener("change", () => updateFilterField(form));
		valueInput.addEventListener("input", () => updateFilterField(form));

		const addButton = document.createElement("input");
		filterRowP.appendChild(addButton);
		addButton.classList.add("add-button");
		addButton.type = "button";
		addButton.value = "+";
		addButton.addEventListener("click", () => addFilterRow(form));

		const removeButton = document.createElement("input");
		filterRowP.appendChild(removeButton);
		removeButton.classList.add("remove-button");
		removeButton.type = "button";
		removeButton.value = "\u2212";
		removeButton.addEventListener("click", () => removeFilterRow(form, filterRowP));

		updateRemoveFilterButtons(form);
		updateFilterField(form);
	}

	function setUp() {
		const form = <HTMLFormElement>document.querySelector("form.icingcake-form");

		const objTypeP = document.createElement("p");
		form.appendChild(objTypeP);
		objTypeP.classList.add("obj-type");

		const objTypeLabel = document.createElement("label");
		objTypeP.appendChild(objTypeLabel);
		objTypeLabel.textContent = "Objekttyp: ";

		const objTypeSelect = document.createElement("select");
		objTypeLabel.appendChild(objTypeSelect);
		objTypeSelect.name = "objtype";

		addSelectOption(objTypeSelect, "Host", "hosts");
		addSelectOption(objTypeSelect, "Service", "services");

		addFilterRow(form);

		const filterField = document.createElement("input");
		form.appendChild(filterField);
		filterField.classList.add("filter-field");
		filterField.type = "hidden";
		filterField.name = "filter";

		const submitP = document.createElement("p");
		form.appendChild(submitP);
		submitP.classList.add("submit");

		const submitButton = document.createElement("input");
		submitP.appendChild(submitButton);
		submitButton.type = "submit";
		submitButton.value = "abfragen";

		const noJsWarning = <HTMLParagraphElement>form.querySelector("p.no-js-warning");
		deleteNode(noJsWarning);
	}

	document.addEventListener("DOMContentLoaded", setUp);
}
