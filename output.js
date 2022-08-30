import ComponentOne from '@/components/ComponentOne.vue';
import ComponentTwo from '@/components/ComponentTwo.vue';
export default defineComponent({
    components: {
        ComponentOne,
        ComponentTwo
    },
    props: {
        id: {
            type: String,
            required: true
        }
    },
    setup (props, ctx) {
        const loading = ref(false);
        const foo = ref(null);
        const count = ref(0);
        const headers = ref([
            {
                text: 'Name',
                value: 'name'
            },
            {
                text: 'Identifier',
                value: 'identifier'
            }, 
        ]);
        foo.value = 1;
        loading.value = true;
        method1.value();
        count.value += 1;
        loading.value = false;
        return {
            loading,
            foo,
            count,
            headers
        };
    }
});
