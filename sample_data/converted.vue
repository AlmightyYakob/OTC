<template>
    <div />
</template>

<script>
// Not real
import ComponentOne from '@/components/ComponentOne.vue';
import ComponentTwo from '@/components/ComponentTwo.vue';
import { defineComponent, ref, inject, onCreated } from '@vue/composition-api';

export default defineComponent({
    components: {
        ComponentOne,
        ComponentTwo,
    },
    props: {
        id: {
            type: String,
            required: true,
        },
    },
    setup() {
        // Inject
        const something = inject('something');

        // Data
        const loading = ref(false);
        const foo = ref(null);
        const count = ref(0);
        const headers = ref([
            {
                text: 'Name',
                value: 'name',
            },
            {
                text: 'Identifier',
                value: 'identifier',
            },
        ]);

        // Created
        foo.value = 1;

        // Mounted
        onMounted(() => {
            loading.value = true;
            something()
            count.value += 1;
            loading.value = false;
        });

        // Methods
        function method1() {
            console.log('nothing!');
        }

        async function method2() {
            console.log('async!')
        }

        return {
            // Inject
            something,

            // Data
            loading,
            foo,
            count,
            headers,

            // Methods
            method1,
            method2,
        }
    },
});
</script>
