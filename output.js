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
        const something = inject('something');
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
        const bar = computed(()=>foo.value || props.id);
        const baz = computed(()=>{
            if (loading.value) {
                return 0;
            }
            return count.value + 1;
        });
        foo.value = 1;
        ctx.$emit('emission');
        function method1(arg) {
            console.log(arg);
        }
        async function method2() {
            console.log('async!');
            something.foo.bar.run();
        }
        onMounted(async ()=>{
            loading.value = true;
            method1(count.value);
            count.value += 1;
            loading.value = false;
        });
        return {
            something,
            loading,
            foo,
            count,
            headers,
            bar,
            baz,
            method1,
            method2
        };
    }
});
