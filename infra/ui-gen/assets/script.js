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
            el.style.display = isVisible ? 'block' : 'none';
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