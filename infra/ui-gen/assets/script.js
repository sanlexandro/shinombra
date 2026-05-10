// Функция для сбора данных из формы с правильными типами
const getFormData = (form) => {
    const formData = new FormData(form);
    const data = {};
    for (let [key, value] of formData.entries()) {
        const parts = key.split(/[\[\]]/).filter(p => p !== "");

        // если строка похожа на число, делаем её числом
        let parsedValue = value;
        if (value !== "" && !isNaN(value)) {
            parsedValue = Number(value);
        }

        if (parts.length === 3) {
            if (!data[parts[0]]) data[parts[0]] = {};
            if (!data[parts[0]][parts[1]]) data[parts[0]][parts[1]] = [];
            data[parts[0]][parts[1]].push(parsedValue);
        } else if (parts.length === 2) {
            if (!data[parts[0]]) data[parts[0]] = {};
            data[parts[0]][parts[1]] = parsedValue;
        } else {
            data[key] = parsedValue;
        }
    }
    form.querySelectorAll('input[type="checkbox"]:not(:disabled)').forEach(cb => {
        const parts = cb.name.split(/[\[\]]/).filter(p => p !== "");
        const val = cb.checked; // true или false
        if (parts.length === 2) {
            if (!data[parts[0]]) data[parts[0]] = {};
            data[parts[0]][parts[1]] = val;
        } else if (parts.length === 1) {
            data[parts[0]] = val;
        }
    });

    return data;
};

// Функция для добавления элементов WrapperVec
window.addWrapperVecItem = function(containerId) {
    const container = document.getElementById(containerId);
    const template = document.getElementById(containerId + '_template');
    const itemsDiv = container.querySelector('.wrapper-vec-items');
    
    // Используем Date.now() чтобы создать уникальный "индекс" для нового элемента
    // На бекенде индекс не важен, т.к. мы используем push для сохранения порядка
    const nextIndex = Date.now(); 
    
    const clone = template.content.cloneNode(true);
    let html = clone.firstElementChild.outerHTML.replace(/__INDEX__/g, nextIndex);
    
    const wrapper = document.createElement('div');
    wrapper.innerHTML = html;
    itemsDiv.appendChild(wrapper.firstElementChild);
};

// Логика управления видимостью (data-show-if)
document.addEventListener('input', () => {
    const form = document.querySelector('form');
    const data = getFormData(form);

    document.querySelectorAll('[data-show-if]').forEach(el => {
        const condition = el.getAttribute('data-show-if');
        try {
            const check = new Function('ctx', `with(ctx) { return ${condition} }`);
            const isVisible = !!check(data);
            el.style.display = isVisible ? '' : 'none';
            el.querySelectorAll('input, select').forEach(input => {
                input.disabled = !isVisible;
            });
        } catch (e) { }
    });
});

// Единый обработчик отправки
document.querySelector('form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const form = e.target;

    if (!form.checkValidity()) {
        alert("Пожалуйста, заполните все активные поля.");
        return;
    }

    const data = getFormData(form);

    const response = await fetch('/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
    });

    if (response.ok) {
        alert("Конфигурация сохранена!");
        window.location.reload();
    } else {
        alert("Ошибка при сохранении");
    }
});

// Первичный запуск логики видимости
document.dispatchEvent(new Event('input'));

// Инициализация пустых значений ползунков при загрузке
document.querySelectorAll('.slider-group').forEach(group => {
    const numInput = group.querySelector('.slider-num');
    const rangeInput = group.querySelector('.slider-range');
    
    if (numInput && rangeInput) {
        // Если у инпута нет value (как у Gamma и Ema), берем дефолтное положение range
        if (!numInput.value) {
            numInput.value = rangeInput.value;
        }
    }
});