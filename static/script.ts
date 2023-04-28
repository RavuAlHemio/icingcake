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
	}

	function addSelectOption(select: HTMLSelectElement, value: string, key?: string|undefined) {
		const opt = document.createElement("option");
		select.appendChild(opt);
		opt.textContent = value;
		if (key !== undefined) {
			opt.value = key;
		}
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

		const operatorSelect = document.createElement("select");
		filterRowP.appendChild(operatorSelect);
		operatorSelect.classList.add("operator");
		addSelectOption(operatorSelect, "entspricht", "match");
		addSelectOption(operatorSelect, "entspricht nicht", "nmatch");
		addSelectOption(operatorSelect, "=", "eq");
		addSelectOption(operatorSelect, "\u2260", "ne");

		const valueInput = document.createElement("input");
		filterRowP.appendChild(valueInput);
		valueInput.type = "text";
		valueInput.classList.add("value");

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

		const noJsWarning = <HTMLParagraphElement>form.querySelector("p.no-js-warning");
		deleteNode(noJsWarning);
	}

	document.addEventListener("DOMContentLoaded", setUp);
}
